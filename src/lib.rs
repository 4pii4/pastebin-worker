#![feature(path_file_prefix)]
use std::{path::Path, ffi::OsStr, panic};

use worker::*;

extern crate console_error_panic_hook;

fn console_error(e: Error) {
    console_error!("{}", e);
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let router = Router::new();
    router
        .get_async("/", |_req, ctx| async move {
            let _result = ctx.kv("rust_worker")
                .map_err(console_error).unwrap()
                .get("/")
                .text().await
                .map_err(|e| console_log!("{}", e)).unwrap()
                .unwrap_or_else(|| "404".to_string());
            Response::from_html(_result)
        })
        .get_async("/:file", |_req, ctx| async move {
            let reqpath = ctx.param("file").unwrap();
            let path = Path::new(reqpath.as_str());
            let name = path.file_prefix().unwrap_or_else(|| OsStr::new("")).to_str().unwrap_or_else(|| "");
            let _result = ctx.kv("rust_worker")
                .map_err(console_error).unwrap()
                .get(name)
                .text().await
                .map_err(|e| console_log!("{}", e)).unwrap()
                .unwrap_or_else(|| "404".to_string());
            return match _result.as_str() {
                "404" => Response::error(_result, 404),
                &_ => {
                    if path.extension() != None {
                        return Response::from_body(
                            ResponseBody::Body(_result.as_str().as_bytes().to_vec())
                        )
                    }
                    Response::ok(_result)
                } 
            }
        })
        .post_async("/", |_req, ctx| async move {
            let mut req_mut = _req.clone_mut().map_err(|e| console_log!("{}", e)).unwrap();
            let form_data = req_mut
                .form_data().await.map_err(console_error).unwrap();
            let form_entry = form_data.get("upload").unwrap_or_else(|| form_data.get("paste").unwrap());
            let file = match form_entry {
                FormEntry::Field(form_entry) => {
                    console_log!("{}", form_entry);
                    File::new(form_entry.into_bytes(), "paste")
                },
                FormEntry::File(form_entry) => {
                    console_log!("{:?}", String::from_utf8(form_entry.bytes().await.unwrap()).unwrap());
                    form_entry
                }
            };
            let filename = file.name();
            let path = Path::new(filename.as_str()).file_prefix().unwrap_or_else(|| OsStr::new("")).to_str().unwrap_or_else(|| "");
            let path_str = path;
            if path_str == "/" {
                return Response::ok("cannot update /")
            }
            let _result = ctx.kv("rust_worker")
                .map_err(console_error).unwrap()
                .put(path_str, String::from_utf8(file.bytes().await.map_err(|e| console_log!("{}", e)).unwrap()).map_err(|e| console_log!("{}", e)).unwrap())
                .map_err(|e| console_log!("{}", e)).unwrap()
                .execute().await;
            let url = _req.url().map_err(console_error).unwrap();
            let redirect = url.to_string() + path_str;
            let redirect_url = Url::parse(redirect.as_str()).unwrap();
            Response::redirect(redirect_url)
        })
        .delete_async("/", |_req, ctx| async move {
            let _result = ctx.kv("rust_worker")
                .map_err(|e| console_log!("{}", e)).unwrap()
                .delete(_req.path().as_str()).await;
            let url = _req.url().unwrap();
            Response::redirect(url)
        })
        .run(req, env).await
}
