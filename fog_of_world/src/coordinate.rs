use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub struct Coordinate {
    pub lon: f64, // 经度
    pub lng: f64, // 纬度
}

impl Coordinate {
    pub fn new(lon:f64, lng:f64)->Self{
        Coordinate{
            lon,
            lng,
        }
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?},{:?}", self.lon, self.lng))
    }
}
impl FromStr for Coordinate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split(",").map(|x| f64::from_str(x).unwrap()).collect::<Vec<f64>>();
        if v.len() < 2 {
            return Err(format!("format error:{:?}", s));
        }
        Ok(Coordinate {
            lon: v[0],
            lng: v[1],
        })
    }
}
