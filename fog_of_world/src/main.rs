use std::fs::File;
use std::io;
use std::time::{Instant};
use fog_of_world::coor_trans::{file_name_to_map_bound};
use io::Read;
use fog_of_world::{amap_api, file_analyze, generate_js};

#[show_image::main]
fn main() {
    let mut points:Vec<generate_js::AllPoints> = vec![];
    let fwss_files = vec![
        ("xiaochai", "snapshots/xiaochai/Snapshot-20241101T213106 0800.fwss", "#08f"),
        ("ox00", "snapshots/ox00/Snapshot-20241104T163651+0800.fwss", "green"),
    ];

    fwss_files.iter().for_each(|(name, file_name, color)|{
        let mut z = zip::ZipArchive::new(File::open(file_name).unwrap()).unwrap();
        let mut each = generate_js::AllPoints{
            name: name.to_string(),
            points: vec![],
            color: color.to_string(),
        };
        let zlen=z.len();
        for i in 0..zlen{
            let f = z.by_index(i).unwrap();
            let zip_each_file_name = f.name().to_string();
            if !zip_each_file_name.starts_with("Model/*/") || !f.is_file(){
                continue
            }

            let (p1,p2) = file_name_to_map_bound(&f.name()[8..]);
            let converts = amap_api::convert_coordinate(vec![&p1,&p2]).unwrap();

            let mut decoder = libflate::zlib::Decoder::new(f).unwrap();
            let mut res = Vec::new();
            decoder.read_to_end(&mut res).unwrap();

            let data = file_analyze::get_full_stream_index(&res);
            let thumb = file_analyze::get_thumb_stream(&res).iter().enumerate()
                .filter(|(x,y)| **y == file_analyze::WHITE)
                .map(|(x,y)|x)
                .collect::<Vec<usize>>();

            let cp1 = converts.get(0).unwrap().as_ref().unwrap();
            let cp2 = converts.get(1).unwrap().as_ref().unwrap();
            println!("get small pic for {:?}, name:{:?} thumb_len:{:?}, small_pic:{:?}, p:{:?}/{:?}", name, zip_each_file_name, thumb.len(), data.len(),  i , zlen);
            let sp = generate_js::SmallPic{
                west_north:vec![cp1.lon, cp1.lng],
                east_south: vec![cp2.lon, cp2.lng],
                data,
                thumb,
            };

            each.points.push(sp);
        }
        points.push(each);
    });

    generate_js::write_2_js_file(points, "./assets/js/data.js").unwrap();
}
