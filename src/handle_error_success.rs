

use std::{ sync::Arc};


use crate::{models::{ SharedState, SuccessErrorMessage}, util::{save_log, send_to_other_server}};


/*
|--------------------------------------------------------------------------
| It send logs and other state to erp if the build process is successful or failed
|--------------------------------------------------------------------------
|
*/

pub async fn handle_error_success(state: &Arc<SharedState>,status:String) {
            

            let mut flag = state.is_building.lock().await;
            if *flag {

                let process_state = state.clone();
                let mut handle = process_state.builder_handle.lock().await;
                if let Some(handle) = handle.take() {
                    handle.abort();
                    *flag = false;

                }
                let config = state.config.lock().await;
                let log_path = config.log_path.clone();

                let buf = state.buffer.lock().await;
                // let json_array = serde_json::to_string(&*buf).unwrap();

                let package_name = state.package_name.lock().await;

                let token = state.token.lock().await;

                // println!("{}",status);

                let is_aborted = status == "aborted";
                let is_error = status != "success";

                let mut url:String = "".to_string();

                if is_error {
                     url = config.on_failure.clone();
                }
                else{
                     url = config.on_success.clone();
                }

                let payload = SuccessErrorMessage{
                    is_aborted,
                    is_error,
                    logs: buf.clone(),
                    package_name: package_name.as_deref().unwrap().to_string(),
                    status,
                    token: token.as_deref().unwrap().to_string()
                };

                
                let json_str = serde_json::to_string_pretty(&payload).unwrap();


                save_log(log_path.clone(),json_str.clone(),token.as_deref().unwrap().to_string()).await;


                tokio::spawn(async move {
                    send_to_other_server(url, json_str).await;
                    println!("Done everything");

                });
            }

            

            /*
            |--------------------------------------------------------------------------
            | Send logs and other state to erp if the build process is successful or failed to ERP here
            |--------------------------------------------------------------------------
            |
            */

            //clear buffer after saving it in file too

            

}
