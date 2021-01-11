use hyper::body::Buf;
use hyper::{header, Body, Request, Response};

use serde::{Deserialize, Serialize};

use gst_pbutils::prelude::*;

use gst_pbutils::DiscovererInfo;
use gst_pbutils::DiscovererStreamInfo;

use anyhow::Error;
use derive_more::{Display, Error};

use std::io::Read;

use std::str;

#[derive(Debug, Display, Error)]
#[display(fmt = "Discoverer error {}", _0)]
struct DiscovererError(#[error(not(source))] &'static str);

#[derive(Serialize, Deserialize)]
struct Discoverer {
    uri: String,
    duration: gst::ClockTime,
    tags: Option<gst::TagList>,
    streams: Vec<Stream>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stream {
    id: Option<String>,
    caps: Option<gst::Caps>,
}

fn print_stream_info(stream: &DiscovererStreamInfo) -> Stream {
    let stream_id = match stream.get_stream_id() {
        Some(id) => Some(id.to_string()),
        _ => None
    };

    Stream{id: stream_id, caps: stream.get_caps()}
}

fn print_discoverer_info(info: &DiscovererInfo) -> Result<Discoverer, Error> {
    let mut streams = Vec::new();
    
    let uri = info
        .get_uri()
        .ok_or(DiscovererError("URI should not be null"))?;
    
    for child in info.get_stream_list() {
        streams.push(print_stream_info(&child));
    }
    
    let res = Discoverer { uri: String::from(uri.as_str()),
                           duration: info.get_duration(),
                           tags: info.get_tags(),
                           streams: streams} ;
    Ok(res)
}

async fn run_discoverer(uri: &str) -> Result<String, Error> {
    gst::init().unwrap();

    let timeout: gst::ClockTime = gst::ClockTime::from_seconds(15);
    let discoverer = gst_pbutils::Discoverer::new(timeout).unwrap();
   
	let info = discoverer.discover_uri(uri).or_else(|err| Err(Error::msg(err)))?;

    match print_discoverer_info(&info) {
            Ok(info) => serde_json::to_string(&info).or_else(|err| Err(Error::msg(err))),
            Err(err) => Err(anyhow::Error::msg(err))
        }
}


pub async fn handle(req: Request<Body>) -> Response<Body> {
    // that can panic do something
    let whole_body = hyper::body::aggregate(req).await.unwrap();
    let mut reader = whole_body.reader();
    let mut dst = [0; 1024];
    let num = reader.read(&mut dst).unwrap();

    let body = str::from_utf8(&dst[0..num]).unwrap();

    match run_discoverer(body).await {
        Ok(v) => Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(v))
        .unwrap(),
        Err(err) => Response::new(Body::from(format!("Error:{}", err)))
    }
}
