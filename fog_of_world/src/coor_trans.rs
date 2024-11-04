use std::collections::{HashMap};
use std::f64::consts::{E, PI};
use std::thread::sleep;
use std::time::Duration;
use lazy_static::lazy_static;
use crate::coordinate::Coordinate;
use crate::file_analyze::test;

lazy_static! {
    /// 文件名的映射
    static ref num_puzzle_map:HashMap<char, usize> = init_num_puzzle_map();
}

/// 初始化数字字母转化的map
fn init_num_puzzle_map() -> HashMap<char, usize> {
    let mut res = HashMap::new();
    "olhwjsktri".chars()
        .enumerate().for_each(|(a, b)| {
        res.insert(b, a);
    });
    res
}

pub fn file_name_to_position(name: &str) -> Coordinate {
    let i = resolve_num_puzzle(name);
    let row = i / 512;
    let col = i % 512;
    let x = 2f64 * PI / 512f64 * (col as f64) - PI;
    let y = PI - 2f64 * PI / 512f64 * (row as f64);
    mercator_rec_2_lon(x, y)
}

/// 将文件名转成第n块数据
pub fn resolve_num_puzzle(name: &str) -> i64 {
    let res: i64 = name[4..name.len() - 2].chars().rev().enumerate().map(
        |(a, b)| {
            let num = *num_puzzle_map.get(&b).expect(
                format!("char {:?} not found in puzzle map!", b).as_str()
            ) as i64;
            10_i64.pow(a as u32) * num
        }
    ).sum();
    res
}

/// 墨卡托地图坐标转化，直角坐标->经纬度
pub fn mercator_rec_2_lon(x: f64, y: f64) -> Coordinate {
    Coordinate::new(x / PI * 180_f64, (E.powf(y).atan() * 2f64 - PI / 2f64) * 180f64 / PI)
}

/// 墨卡托地图坐标转化，经纬度->直角坐标
///  longitude经度，latitude纬度
pub fn mercator_lon_2_rec(x: Coordinate) -> (f64, f64) {
    (x.lon / 180f64 * PI, (x.lng / 180f64 * PI / 2f64 + PI / 4f64).tan().abs().ln())
}

// 根据文件名给出渲染矩形需要的bound，即左上角和右下角的坐标
pub fn file_name_to_map_bound(name: &str) -> (Coordinate, Coordinate) {
    let i = resolve_num_puzzle(name);
    let row = i / 512;
    let col = i % 512;
    let x = 2f64 * PI / 512f64 * (col as f64) - PI;
    let y = PI - 2f64 * PI / 512f64 * (row as f64);
    let x_1 = 2f64 * PI / 512f64 * ((col + 1) as f64) - PI;
    let y_1 = PI - 2f64 * PI / 512f64 * ((row + 1) as f64);
    // println!("{:?},{:?}, {:?}", x_1-x, y_1-y, 2f64 * PI / 512f64);
    (mercator_rec_2_lon(x, y), mercator_rec_2_lon(x_1, y_1))
}