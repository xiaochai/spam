use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use anyhow::anyhow;
use serde_json::Value;
use tracing::log::__private_api::loc;
use tracing_subscriber::fmt::format;
use crate::file_cache::FileCache;
use crate::coordinate::Coordinate;

const AMAP_KEY: &str = "92109bc80d3baf721bf5ed28f29c73ec";

lazy_static::lazy_static! {
    static ref HTTP_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref FILE_CACHE:Arc<Mutex<FileCache>> = Arc::new(Mutex::new(FileCache::new("cache/file_cache_coordinate")));
}

// 转为高德坐标，即WGS-84 -> GCJ-02，参数为(经度,纬度)的vec
pub fn convert_coordinate(locations: Vec<&Coordinate>) -> anyhow::Result<Vec<Option<Coordinate>>> {
    let mut q_location: String; // 需要进行远程查询的经纬度坐标参数，经纬度以逗号分隔，多个以|分隔
    let mut ret_val: Vec<Option<Coordinate>> = Vec::new(); // 最终返回结果
    {
        let binding = FILE_CACHE.lock().unwrap();
        let mut q_arr = Vec::new();
        locations.iter().for_each(
            |x| {
                let t = binding.get::<Coordinate, Coordinate>(*x);
                if t.is_none() { q_arr.push(format!("{:?},{:?}", x.lon, x.lng)) };
                ret_val.push(t)
            }
        );
        q_location = q_arr.join("|");
    }
    // 如果全部命中文件缓存，则直接返回
    if q_location == "" {
        return Ok(ret_val);
    }

    // 请求远程
    let url = String::new() +
        "https://restapi.amap.com/v3/assistant/coordinate/convert?key=" +
        AMAP_KEY +
        "&locations=" + &q_location +
        "&coordsys=gps";
    let mut response = HTTP_CLIENT.get(url).send()?;
    println!("statusCode:{:?}", response.status());
    let mut resp_str = String::new();
    response.read_to_string(&mut resp_str)?;
    println!("get convert res:{:?}", resp_str.as_str());
    let json: Value = serde_json::from_str(resp_str.as_str())?;
    let arr = json.as_object().ok_or(anyhow!("convert raw response is not obj"))?
        .get("locations").ok_or(anyhow!("convert locations is empty"))?
        .as_str().ok_or(anyhow!("convert locations is not str"))?;
    let resp_coors = arr.split(";")
        .map(|x| Coordinate::from_str(x).unwrap())
        .collect::<Vec<Coordinate>>();
    {
        let mut binding = FILE_CACHE.lock().unwrap();
        let mut resp_coors_index = 0; // 用于记录resp处理到哪了
        for (i, c) in locations.iter().enumerate() {
            let from_fc = ret_val.get(i).unwrap();
            if from_fc.is_some() { continue; }; // 如果在文件缓存中命中了，那么直接跳过
            let from_remote = resp_coors.get(resp_coors_index);
            match from_remote {
                None => {}
                Some(x) => {
                    binding.set(*c, x); // 记录文件
                    ret_val[i] = Some(Coordinate::new(x.lon, x.lng)); // 添加进数组
                    resp_coors_index += 1;
                }
            }
        }
    }
    Ok(ret_val)
}
