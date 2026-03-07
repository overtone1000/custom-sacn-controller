use std::{collections::BTreeMap, net::SocketAddr, sync::{Arc, RwLock}};

use hyper::{body::Incoming, Method, Request, Response};
use hyper_services::{
    commons::HandlerResult, request_processing::get_request_body_as_string, response_building::{bad_request, bytes_to_boxed_body}, service::stateful_service::StatefulHandler
};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::pixels::{pixelstripcommand::PixelStripCommand, pixelstripmanager::PixelStripManager};

#[derive(Clone)]
pub struct LedCommandHandler {
    pixel_strip_manager: Arc<PixelStripManager>,
}

impl  LedCommandHandler {
    pub fn new(pixel_strip_manager:Arc<PixelStripManager>) -> LedCommandHandler {
        LedCommandHandler {
            pixel_strip_manager,
        }
    }
}

impl  StatefulHandler for LedCommandHandler {

    async fn handle_request(self: Self, request: Request<Incoming>) -> HandlerResult {
        let method: hyper::Method = request.method().clone();
        //let path = request.uri().path().to_string();
        
        match method {
            Method::POST => {
                let body= match get_request_body_as_string(request.into_body()).await
                {
                    Ok(body)=>body,
                    Err(e)=>{
                        eprintln!("Couldn't get request body. {:?}",e);
                        return Ok(bad_request());
                    }
                };

                let command:PixelStripCommand=match serde_json::from_str(&body){
                    Ok(body)=>body,
                    Err(e)=>{
                        eprintln!("Couldn't deserialize pixel strip command. {:?}",e);
                        eprintln!("Received string was: {}",body);
                        return Ok(bad_request());
                    }
                };

                println!("Received {:?}",command);

                {
                    self.pixel_strip_manager.queue_command(command);
                }

                return Ok(Response::new(bytes_to_boxed_body("Ok")));
            },
            method=>{
                eprintln!("Received unexpected method {:?}",method);
                return Ok(bad_request());
            }
        }
        
        //return Ok(not_found());
        //return Ok(bad_request());

        
    }
}