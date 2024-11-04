use std::collections::{HashMap};
use std::f64::consts::{E, PI};
use std::fs::File;
use std::io;
use std::ptr::copy;
use std::thread::sleep;
use std::time::Duration;
use lazy_static::lazy_static;
use crate::coordinate::Coordinate;
use crate::coor_trans::{mercator_rec_2_lon, file_name_to_map_bound};
use io::Read;
use std::io::Write;
use serde::{Deserialize, Serialize};
use crate::{amap_api, file_analyze};

#[derive(Serialize, Deserialize)]
pub struct SmallPic {
    pub west_north:Vec<f64>,
    pub east_south:Vec<f64>,
    pub data:Vec<usize>,
    pub thumb:Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct AllPoints{
    pub name:String,
    pub points: Vec<SmallPic>,
    pub color: String,
}

pub fn write_2_js_file(points:Vec<AllPoints>, file_name:&str)->anyhow::Result<()>{
    let json = serde_json::to_string(&points)?;
    let mut wf = File::create(file_name)?;
    wf.write("const points = ".as_bytes())?;
    wf.write(json.as_bytes())?;
    Ok(())
}
