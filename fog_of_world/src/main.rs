use std::collections::{HashMap};
use std::f64::consts::{E, PI};
use std::fs::File;
use std::io;
use std::ptr::copy;
use std::thread::sleep;
use std::time::Duration;
use lazy_static::lazy_static;
use fog_of_world::coordinate::Coordinate;
use fog_of_world::coor_trans::{mercator_rec_2_lon, file_name_to_map_bound};
use io::Read;
use std::io::Write;
use serde::{Deserialize, Serialize};
use fog_of_world::{amap_api, file_analyze, generate_js};

#[show_image::main]
fn main() {
    let mut points:Vec<generate_js::AllPoints> = vec![];

    let file_name="snapshots/xiaochai/Snapshot-20241101T213106 0800.fwss";
    let mut z = zip::ZipArchive::new(File::open(file_name).unwrap()).unwrap();
    let mut each = generate_js::AllPoints{
        name: "xiaochai".to_string(),
        points: vec![],
        color: "red".to_string(),
    };

    for i in 0..z.len(){
        let f = z.by_index(i).unwrap();
        if !f.name().starts_with("Model/*/") || !f.is_file(){
            continue
        }

        let (p1,p2) = file_name_to_map_bound(&f.name()[8..]);
        let converts = amap_api::convert_coordinate(vec![&p1,&p2]).unwrap();
        // println!("{:?}", converts);


        let mut decoder = libflate::zlib::Decoder::new(f).unwrap();
        let mut res = Vec::new();
        decoder.read_to_end(&mut res).unwrap();
        let image_v = file_analyze::get_full_stream(&res);

       let data = image_v.iter().enumerate()
            .filter(|(x, y)| **y == file_analyze::WHITE)
            .map(|(x, y)| x)
            .collect::<Vec<usize>>();

        let cp1 = converts.get(0).unwrap().as_ref().unwrap();
        let cp2 = converts.get(1).unwrap().as_ref().unwrap();
        let sp = generate_js::SmallPic{
            west_north:vec![cp1.lon, cp1.lng],
            east_south: vec![cp2.lon, cp2.lng],
            data,
        };

        each.points.push(sp);

        let image_width = (file_analyze::THUMB_WIDTH_HEIGHT * file_analyze::SMALL_PIC_WIDTH_HEIGHT) as u32;
        file_analyze::image_show(image_width, image_width, image_v);
        sleep(Duration::from_secs(5));

    }
    points.push(each);
    generate_js::write_2_js_file(points, "./assets/js/data.js").unwrap();
    return;


    println!("{:?}", fog_of_world::file_analyze::test("0921iihwtxn"));
    return;
    let t = fog_of_world::amap_api::convert_coordinate(
        vec![&Coordinate{lon:116.01562499999999,lng:39.90973623453719},
             &Coordinate{lon:116.71875000000001,lng:40.44694705960048}
        ],)
        .expect("TODO: panic message");
    println!("{:?}", t);
    return;

    println!("{:?}, {:?}", mercator_rec_2_lon(PI,PI), file_name_to_map_bound("0921iihwtxn"));
    return ;
    println!("{:?}, {:?}", mercator_rec_2_lon(PI,PI), file_name_to_map_bound("0921iihwtxn"));
    // println!("{:?}, {:?}", mercator_rec_2_lon(PI,PI), sync_file_name_to_position("8e7clljsiwox"))
    sleep(Duration::from_secs(10))
}
