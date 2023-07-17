use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let method = req.method().to_string();
    let method_str = method.as_str();
    let mut req_mut = req.clone_mut().map_err(|e| console_log!("{}", e)).unwrap();
    match method_str {
        "GET" => {
            let _result = env.kv("rust_worker")
                .map_err(|e| console_log!("{}", e)).unwrap()
                .get(req.path().as_str())
                .text().await
                .map_err(|e| console_log!("{}", e)).unwrap()
                .unwrap_or_else(|| "404".to_string());
            return match _result.as_str() {
                "404" => Response::error(_result, 404),
                &_ => {
                    if req.path().as_str() == "/" {
                        return Response::from_html(_result)
                    }
                    if req.path().as_str().trim_end_matches(".*").len() > 0 {
                        return Response::from_body(
                            ResponseBody::Body(_result.as_str().as_bytes().to_vec())
                        )
                    }
                    Response::ok(_result)
                } 
            }
        }
        "POST" | "PUT" => {
            if req.path().as_str() == "/" {
                return Response::ok("cannot update /")
            }
            let form_entry = req_mut
                .form_data().await.map_err(|e| console_log!("{}", e)).unwrap()
                .get("upload").unwrap();
            let file = match form_entry {
                FormEntry::Field(form_entry) => {
                    console_log!("{}", form_entry);
                    File::new(form_entry.into_bytes(), "upload")
                },
                FormEntry::File(form_entry) => {
                    console_log!("{:?}", form_entry.bytes().await);
                    form_entry
                }
            };
            let _result = env.kv("rust_worker")
                .map_err(|e| console_log!("{}", e)).unwrap()
                .put(req.path().as_str(), String::from_utf8(file.bytes().await.unwrap()).unwrap())
                .map_err(|e| console_log!("{}", e)).unwrap()
                .execute().await;
            Response::ok(String::from_utf8(file.bytes().await.unwrap()).unwrap())
        },
        "DELETE" => {
            let _result = env.kv("rust_worker")
                .map_err(|e| console_log!("{}", e)).unwrap()
                .delete(req.path().as_str()).await;
            Response::ok("404")
        }
        &_ => Response::ok(method)
    }
}
