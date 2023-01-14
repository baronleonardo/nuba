use std::{net::SocketAddr, str::FromStr, error::Error, fs::{File, OpenOptions}, collections::HashMap, io::{Read, Write, Seek, SeekFrom}, sync::Arc};
use hyper::{server::conn::Http, service::service_fn, Request, Body, Response, Method, StatusCode};
use tokio::{net::TcpListener, sync::{Mutex, MutexGuard}};
use diffy::{Patch, apply_bytes};

type FilesDb = Arc<Mutex<HashMap<String, File>>>;

pub struct Server
{
    listener: TcpListener,
    opened_files: FilesDb
}

impl Server
{
    pub async fn new(addr: &str) -> Result<Server, Box<dyn std::error::Error + Sync + Send + 'static>>
    {
        let socketaddr = SocketAddr::from_str(addr)?;
        let listener = TcpListener::bind(socketaddr).await?;
        println!("Listening on http://{}", addr);

        Ok(Server { listener, opened_files: Arc::new(Mutex::new(HashMap::new())) })
    }

    pub async fn run(&self) -> tokio::io::Result<()>
    {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let opened_files = self.opened_files.clone();
    
            tokio::task::spawn(async move {
                if let Err(err) = Http::new()
                    .serve_connection(
                        stream,
                        service_fn(move |req| Self::r#impl(req, opened_files.clone())))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    async fn r#impl(req: Request<Body>, opened_files: FilesDb) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        // REMOVE
        // println!("{:?}", req.uri().query().unwrap().split('&').collect::<Vec<&str>>().join(" "));

        let uri = req.uri().clone();
        let query = uri.query();
        if let None = query
        { 
            return Self::response_error(StatusCode::NOT_ACCEPTABLE, "Empty query");
        }
        let path = query.unwrap();

        match (req.method(), req.uri().path())
        {
            (&Method::GET, "/create_file")  => Self::on_create_file(path).await,
            (&Method::GET, "/create_dir")   => Self::on_create_dir(path).await,
            (&Method::GET, "/remove")       => Self::on_remove(path, opened_files).await,
            (&Method::GET, "/close")        => Self::on_close(path, opened_files).await,
            (&Method::GET, "/read")         => Self::on_read(path, opened_files).await,
            (&Method::POST, "/write")       => Self::on_write(path, req.into_body(), opened_files).await,

            // Return the 404 Not Found for other routes.
            _ => Self::response_error(StatusCode::NOT_FOUND, "Unsupported route")
        }
    }

    async fn on_create_file(path: &str) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        match std::fs::File::create(path)
        {
            Ok(_) => Self::response_ok(&format!("GET created file {}", path)),
            Err(e) => Self::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
        }
    }

    async fn on_create_dir(path: &str) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        match std::fs::create_dir_all(path)
        {
            Ok(_) => Self::response_ok(&format!("GET created dir {}", path)),
            Err(e) => Self::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
        }
    }

    async fn on_read(path: &str, opened_files: FilesDb) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        let mut opened_files = opened_files.lock().await;

        match opened_files.get(path)
        {
            Some(mut f) => {
                let mut buf = Vec::new();
                f.seek(SeekFrom::Start(0))?;
                f.read_to_end(&mut buf)?;
                Ok(Response::builder()
                            .body(Body::from(buf))
                            .unwrap())
            },
            None => {
                let mut f = Self::open_file(path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;

                match std::fs::read(path)
                {
                    Ok(data) => {
                        opened_files.insert(path.to_owned(), f);
                        Ok(Response::builder().body(Body::from(data)).unwrap())
                    },
                    Err(e) => Self::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
                }
            }
        }
    }

    /// I recieve patch file and apply it to the file
    async fn on_write(path: &str, body: Body, opened_files: FilesDb) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        let mut opened_files = opened_files.lock().await;
        let bytes = hyper::body::to_bytes(body).await?;
        let patch = Patch::from_bytes(&bytes).unwrap();

        async fn impl_apply_bytes(file_data :&Vec<u8>, patch: &Patch<'_, [u8]>, path: &str, opened_files: &mut MutexGuard<'_, HashMap<String, File>>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
        {
            match apply_bytes(&file_data, &patch)
            {
                Ok(patched_result) => {
                    let mut f = Server::open_file(path)?;
                    match f.write_all(&patched_result)
                    {
                        Ok(_) => {
                            opened_files.insert(path.to_owned(), f);
                            Server::response_ok(&format!("POST write to file {}", path))
                        },
                        Err(e) => Server::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
                    }
                },
                Err(e) => Server::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
            }
        }

        match opened_files.get(path)
        {
            Some(mut f) => {
                let mut buf = Vec::<u8>::new();
                f.seek(SeekFrom::Start(0))?;
                f.read_to_end(&mut buf)?;
                impl_apply_bytes(&buf, &patch, &path, &mut opened_files).await
            },
            None => {
                let file_data_status = std::fs::read(path);
                if let Err(e) = file_data_status {
                    return Self::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string());
                }
                let file_data = file_data_status.unwrap();
                impl_apply_bytes(&file_data, &patch, &path, &mut opened_files).await
            }
        }
    }

    async fn on_remove(path: &str, opened_files: FilesDb) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        // close the file first
        let mut opened_files = opened_files.lock().await;
        if let Some(_) = opened_files.get(path)
        {
            opened_files.remove(path);
        }

        match std::fs::metadata(path)
        {
            Ok(metadata) => {
                let filetype = metadata.file_type();

                if filetype.is_dir()
                {
                    match std::fs::remove_dir_all(path)
                    {
                        Ok(_) => Self::response_ok(&format!("GET remove directory {}", path)),
                        Err(e) => Self::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
                    }
                }
                else if filetype.is_file()
                {
                    match std::fs::remove_file(path)
                    {
                        Ok(_) => Self::response_ok(&format!("GET remove file {}", path)),
                        Err(e) => Self::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
                    }
                }
                else
                {
                    unreachable!()
                }
            },
            Err(e) => Self::response_error(StatusCode::NOT_ACCEPTABLE, &e.to_string())
        }
    }

    async fn on_close(path: &str, opened_files: FilesDb) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        let mut opened_files = opened_files.lock().await;

        match opened_files.get(path)
        {
            Some(_) => {
                opened_files.remove(path);
                Self::response_ok(&format!("GET close file {}", path))
            },
            None => Self::response_error(StatusCode::NOT_ACCEPTABLE, "Not exists")
        }
    }

    fn open_file(path: &str) -> std::io::Result<File>
    {
        OpenOptions::new().write(true).read(true).create(true).open(path)
    }

    fn response_error(statuscode: StatusCode, error_msg: &str) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        Ok(Response::builder()
            .status(statuscode)
            .body(Body::from(error_msg.to_owned()))
            .unwrap())
    }

    fn response_ok(msg: &str) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>
    {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(msg.to_owned()))
            .unwrap())
    }
}