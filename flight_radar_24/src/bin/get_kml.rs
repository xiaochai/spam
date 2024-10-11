use std::fs::File;
use std::{fmt, fs, io};
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Split, Write};
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::anyhow;
use chrono::{DateTime, Days, Local, NaiveDateTime};
use yaml_rust::{YamlLoader, YamlEmitter, Yaml, ScanError};
use reqwest;
use reqwest::header::HeaderMap;
use reqwest::{blocking, cookie, multipart, StatusCode};
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::{Map, Value};

// 不确定复用行不行？
lazy_static::lazy_static! {
    static ref HTTP_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取配置内容
    let c = load_application_config()?;
    // 解析配置
    let doc = &c[0];
    let user_name = match doc["account"]["user_name"].as_str() {
        None => panic!("user_name empty"),
        Some(t) => t,
    };
    let passport = match doc["account"]["password"].as_str() {
        None => panic!("password empty"),
        Some(t) => t,
    };
    let cookie_file = doc["account"]["cookie_file"].as_str().ok_or(anyhow!("cookie_file config empty"))?;
    let resp_file = doc["account"]["resp_file"].as_str().ok_or(anyhow!("cookie_file config empty"))?;
    print!("config info:user_name:{:?}, passport:{:?}, cookie_file:{:?}, resp_file:{:?}\n", user_name, passport, cookie_file, resp_file);

    // 登录
    login(user_name, passport, cookie_file, resp_file)?;

    // 获取航班信息
    let flight = get_flight_by_query_date(cookie_file, "zh9114", "2024-8-30")?;
    println!("find flight{:?}", flight);

    // 下载文件
    let kml_file = download(cookie_file, &flight)?;
    println!("get kml file:{:?}", kml_file);
    Ok(())
}

fn download(cookie_file: &str, flight: &Flight) -> anyhow::Result<String> {
    // 获取起飞的日期
    let datetime = DateTime::from_timestamp(flight.real_departure, 0)
        .ok_or(anyhow!("parse real_departure to datetime failed"))?
        .with_timezone(&Local)
        .format("%Y_%m_%d").to_string();
    // 确定文件名
    let file_name = "./src/config/flight_kml_".to_string() + flight.number.as_str() + "_" + datetime.as_str() + ".kml";
    // 文件是否已经存在
    match fs::metadata(&file_name) {
        Ok(meta) => return Ok(file_name),
        Err(e) => println!("check kml file:{:?} failed:{:?}", file_name, e),
    };

    // 下载文件
    let cookie = fs::read_to_string(cookie_file)?;
    let url = "https://www.flightradar24.com/download/?file=kml&trailLimit=0&flight=".to_string() + flight.id.as_str() + "&history=" + flight.scheduled_departure.to_string().as_str();
    println!("download url:{:?}", url);
    let mut res = HTTP_CLIENT.get(url)
        .header("cookie", cookie.as_str())
        .send()?;
    println!("statusCode:{:?}", res.status());
    let mut output_file = File::create(&file_name)?;
    std::io::copy(&mut res, &mut output_file)?;

    Ok(file_name)
}

#[derive(Debug)]
struct Flight {
    id: String,
    number: String, // 航班名称
    real_departure: i64,
    real_arrival: i64,
    scheduled_departure: i64,
    scheduled_arrival: i64,
}

fn get_flight_by_query_date(cookie_file: &str, query: &str, date: &str) -> anyhow::Result<Flight> {
    let token = get_token_from_cookie(cookie_file)?;
    let datetime: DateTime<Local> = chrono::DateTime::from_str((date.to_string() + " 23:59:59+08:00").as_str())?;
    let mut query_date: i64 = datetime.timestamp();
    if datetime.checked_add_days(Days::new(60)).ok_or(anyhow!("add days error"))?.timestamp()
        < std::time::SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64
    {
        // 如果是60天之前的日期，则不再拉取之前的数据，直接使用最新的
        query_date = 0;
    }
    println!("get_flight_by_query_date query:{:?}, date:{:?}", query, query_date);

    let list = get_flight_list(query, token.as_str())?;
    let res: Value = serde_json::from_str(list.as_str())?;
    for i in get_flight_list_real_data(&res)? {
        let flight = match get_flight_each_data(i) {
            Ok(f) => f,
            Err(e) => {
                println!("get flight but not ok:{:?}, skip...", e.to_string());
                continue;
            }
        };
        if flight.number.to_lowercase() != query.to_lowercase() {
            continue;
        }
        if query_date == 0 ||
            (query_date - flight.real_departure > 0 && query_date - flight.real_departure < 86400) {
            return Ok(flight);
        } else {
            continue;
        }
    }

    Err(anyhow!("flight not found"))
}

fn get_flight_list_real_data(v: &Value) -> anyhow::Result<&Vec<Value>> {
    let arr = v.as_object().ok_or(anyhow!("flight raw response is not obj"))?
        .get("result").ok_or(anyhow!("flight result is empty"))?
        .as_object().ok_or(anyhow!("flight result is not obj"))?
        .get("response").ok_or(anyhow!("flight response is empty"))?
        .as_object().ok_or(anyhow!("flight response is not obj"))?
        .get("data").ok_or(anyhow!("flight data is empty"))?
        .as_array().ok_or(anyhow!("flight result is not array"))?;
    Ok(arr)
}

fn get_flight_each_data(v: &Value) -> anyhow::Result<Flight> {
    let identification = v.get("identification").ok_or(anyhow!("each flight identification is empty"))?
        .as_object().ok_or(anyhow!("each flight identification is not obj"))?;
    let id = identification.get("id").ok_or(anyhow!("each flight identification.id is empty"))?
        .as_str().ok_or(anyhow!("each flight identification.id is not string"))?;
    let number = identification.get("number").ok_or(anyhow!("each flight identification.number is empty"))?
        .as_object().ok_or(anyhow!("each flight identification.number is not obj"))?
        .get("default").ok_or(anyhow!("each flight identification.number.default is empty"))?
        .as_str().ok_or(anyhow!("each flight identification.number.default is not string"))?;


    let time = v.get("time").ok_or(anyhow!("each flight time is empty"))?
        .as_object().ok_or(anyhow!("each flight time is not obj"))?;
    let real = time.get("real").ok_or(anyhow!("each flight time.real is empty"))?
        .as_object().ok_or(anyhow!("each flight time.real is not string"))?;
    let real_departure = real.get("departure").ok_or(anyhow!("each flight time.real.departure is empty"))?
        .as_i64().ok_or(anyhow!("each flight time.real.departure is not i64"))?;
    let real_arrival = real.get("arrival").ok_or(anyhow!("each flight time.real.arrival, is empty"))?
        .as_i64().ok_or(anyhow!("each flight time.real.arrival, is not i64"))?;

    let scheduled = time.get("scheduled").ok_or(anyhow!("each flight time.scheduled is empty"))?
        .as_object().ok_or(anyhow!("each flight time.scheduled is not string"))?;
    let scheduled_departure = real.get("departure").ok_or(anyhow!("each flight time.scheduled.departure is empty"))?
        .as_i64().ok_or(anyhow!("each flight time.scheduled.departure is not i64"))?;
    let scheduled_arrival = real.get("arrival").ok_or(anyhow!("each flight time.scheduled.arrival, is empty"))?
        .as_i64().ok_or(anyhow!("each flight time.scheduled.arrival, is not i64"))?;

    Ok(Flight {
        id: id.to_string(),
        number: number.to_string(),
        real_departure,
        real_arrival,
        scheduled_departure,
        scheduled_arrival,
    })
}


/// 请求远程获取航班列表
fn get_flight_list(query: &str, token: &str) -> anyhow::Result<String> {
    let file_name = "./src/config/flight_".to_string() + query + "_" + Local::now().format("%Y_%m_%d").to_string().as_str();
    match fs::read_to_string(&file_name) {
        Ok(body) => return Ok(body),
        Err(e) => println!("get flight list from file:{:?} failed:{:?}", file_name, e),
    };

    let url = "https://api.flightradar24.com/common/v1/flight/list.json?query=".to_string() + query + "&fetchBy=flight&limit=100&token=" + token;
    let resp = HTTP_CLIENT.get(url).send()?;
    let body = resp.text()?;

    match File::create(&file_name) {
        Ok(mut f) => {
            match f.write_all(body.as_ref()) {
                Ok(_) => {}
                Err(e) => println!("write all to file:{:?} failed:{:?}", file_name, e)
            }
        }
        Err(e) => println!("create file:{:?} failed:{:?}", file_name, e)
    }

    Ok(body)
}

fn get_token_from_cookie(cookie_file: &str) -> anyhow::Result<String> {
    let res = fs::read_to_string(cookie_file)?;
    let f = res.split(";").find(|x| x.starts_with("_frPl")).ok_or(anyhow!("token not found"))?;

    Ok(f[6..].to_string())
}


/// 检查指定文件是否存在，并且修改时间在一天内
/// 如果是，则认为有效返回true，否则false
fn record_file_valid(f: &str) -> bool {
    match fs::metadata(f) {
        Ok(m) => {
            match m.modified() {
                Ok(t) => {
                    SystemTime::now().duration_since(t).unwrap() < Duration::from_secs(86400)
                }
                Err(_) => false
            }
        }
        Err(_) => { false }
    }
}

/// 检查cookie和resp是否存在，如果不存在，走一下登录流程，如果存在，则不做任何事情
fn login(user_name: &str, passport: &str, cookie_file_name: &str, response_file_name: &str) -> anyhow::Result<()> {
    // 检查文件合法性，确认是否需要重新登录
    if record_file_valid(cookie_file_name) && record_file_valid(response_file_name) {
        println!("file exist, skip login");
        return Ok(());
    }
    let (code, body, cookie) = login_request(user_name, passport)?;
    println!("login get cookie:{:?}， resp:{:?}", cookie, body);

    File::create(cookie_file_name)?.write_all(cookie.as_ref())?;
    File::create(response_file_name)?.write_all(body.as_ref())?;
    Ok(())
}


/// 发起远程登录请求
fn login_request(user_name: &str, passport: &str) -> Result<(StatusCode, String, String), reqwest::Error> {
    let url = "https://www.flightradar24.com/user/login";
    // let url = "https://www.douyin.com";
    let mut headers = HeaderMap::new();
    // headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36".parse().unwrap());
    let form = blocking::multipart::Form::new()
        .text("email", user_name.to_string())
        .text("password", passport.to_string());

    let res = HTTP_CLIENT
        .post(url)
        .headers(headers)
        .multipart(form)
        .send()?;
    let res_status = res.status();
    let mut cookie_str = String::new();
    for c in res.cookies() {
        cookie_str = cookie_str + c.name() + "=" + c.value() + ";";
    };
    Ok((res_status, res.text()?, cookie_str))
}

#[derive(Debug)]
enum LoadApplicationConfigErr {
    IOError(io::Error),
    ScanError(ScanError),
}

impl From<io::Error> for LoadApplicationConfigErr {
    fn from(value: io::Error) -> Self {
        LoadApplicationConfigErr::IOError(value)
    }
}
impl From<ScanError> for LoadApplicationConfigErr {
    fn from(value: ScanError) -> Self {
        LoadApplicationConfigErr::ScanError(value)
    }
}

impl Display for LoadApplicationConfigErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for LoadApplicationConfigErr {}

fn load_application_config() -> Result<Vec<Yaml>, LoadApplicationConfigErr> {
    let file_name = "./src/config/application.yml";
    let mut content = String::new();
    File::open(file_name)?.read_to_string(&mut content)?;
    Ok(YamlLoader::load_from_str(&content)?)
}