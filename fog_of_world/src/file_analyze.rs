use std::fs::File;
use std::hint::black_box;
use std::io::Read;
use std::thread::sleep;
use std::time::{Duration, Instant};
use image::{ExtendedColorType, Luma};
use show_image::{ImageInfo, ImageView, create_window, WindowOptions};
use tracing_subscriber::fmt::time;

// 对于返回数据流结果黑底，白的路径，如果要改这个，请改WHITE和BLACK的定义
// 所以对于需要只处理有内容的点时，判断此点不为0即可
const BLACK: u8 = 0;
pub const WHITE: u8 = 255;

/// 存档的单个文件格式说明
/// 首先一个文件包含有8192*8192块内容，而这8129*8192又分成128*128的小块，每块为64*64
/// 在文件的形状，会有128*128个2字节用于编号有点的小块位置，这个编号可以索引这个小块的内容在文件剩下部分的位置
/// 例如一个文件有中有n个小块有内容，则文件剩下部分为(64*64/8+3)*n，除以8是因为按bit存储，+3是因为多了3字节
/// 所以我们最终文件分成前面的128*128*2字节的缩略图(thumb)部分，和后面(64*64/8+3)*n小图(small_pic)部分

/// 缩略图长宽和字节数量
pub const THUMB_WIDTH_HEIGHT: usize = 128;
const THUMB_LEN: usize = THUMB_WIDTH_HEIGHT * THUMB_WIDTH_HEIGHT * 2;

/// 小图长宽和字节数量，小图按位存储，所以要除以8，3字节为额外字节，不明含义
pub const SMALL_PIC_WIDTH_HEIGHT: usize = 64;
const SMALL_PIC_LEN: usize = SMALL_PIC_WIDTH_HEIGHT * SMALL_PIC_WIDTH_HEIGHT / 8 + 3;

/// 从一个zlib压缩文件中获取二进制流
/// 由于存档文件是压缩的，所以需要先解压
pub fn get_zlib_file_bytes(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let file = File::open(file_name)?;
    let mut decoder = libflate::zlib::Decoder::new(file)?;
    let mut res = Vec::new();
    decoder.read_to_end(&mut res)?;
    Ok(res)
}

/// 从缩略图中解析出来128*128的数组，每个元素表示这个位置有没有点，有点的话，表示这块在文件剩余部分的位置
pub fn get_thumb_indicate(c: &Vec<u8>) -> Vec<u16> {
    c.iter().take(THUMB_LEN).enumerate().step_by(2).map(
        |(i, x)| {
            (*x as u16) + (*c.get(i + 1).unwrap() as u16) * 16 * 16
        }
    ).collect()
}

/// 获取缩略图可用于展示的数据流，0表示没有数据，255表示有数据
pub fn get_thumb_stream(c: &Vec<u8>) -> Vec<u8> {
    c.iter().take(THUMB_LEN).enumerate().step_by(2).map(
        |(i, x)| {
            if *x > 0 || *c.get(i + 1).unwrap() > 0 { WHITE } else { BLACK }
        }
    ).collect()
}


// 获取第N张有内容的小图的像素点，返回的大小为SMALL_PIC_WIDTH_HEIGHT*SMALL_PIC_WIDTH_HEIGHT
fn get_small_pic_with_pos(c: &Vec<u8>, n: u16) -> Vec<u8> {
    c.iter()
        // 跳过缩略图和n-1张小图
        .skip(THUMB_LEN + SMALL_PIC_LEN * (n - 1) as usize)
        // 只处理第n+1张图的部分
        .take(SMALL_PIC_WIDTH_HEIGHT * SMALL_PIC_WIDTH_HEIGHT / 8)
        .map(
            // 将字节展开，因为这个部分是拿bit来存储信息的
            |x| {
                let mut tmp: Vec<u8> = vec![];
                if 0x80 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                if 0x40 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                if 0x20 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                if 0x10 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                if 0x8 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                if 0x4 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                if 0x2 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                if 0x1 & x > 0 { tmp.push(WHITE) } else { tmp.push(BLACK) }
                tmp
            }
        ).flatten().collect()
}

/// 简易展示图片的工具
pub fn image_show(width: u32, height: u32, v: Vec<u8>) {
    let image = ImageView::new(ImageInfo::mono8(width, height), v.as_slice());
    let option = WindowOptions::new().set_fullscreen(false);
    let window = create_window("image", option).unwrap();
    window.set_image("image-001", image).unwrap();
}

/// 获取整张图的像素点
pub fn get_full_stream(c: &Vec<u8>) -> Vec<u8> {
    let mut res = vec![BLACK; (THUMB_WIDTH_HEIGHT*SMALL_PIC_WIDTH_HEIGHT).pow(2)];
    get_thumb_indicate(&c).iter().enumerate().filter(|(_,x)| **x > 0).for_each(
        |(i,x)| {
            get_small_pic_with_pos(c, *x).iter().enumerate().filter(|(_,x)| **x > 0).for_each(
                |(ii, xx)|{
                    let col = (i % THUMB_WIDTH_HEIGHT)*SMALL_PIC_WIDTH_HEIGHT+ii%SMALL_PIC_WIDTH_HEIGHT;
                    let row = (i/THUMB_WIDTH_HEIGHT)*SMALL_PIC_WIDTH_HEIGHT+ii/SMALL_PIC_WIDTH_HEIGHT;
                    res[row*THUMB_WIDTH_HEIGHT*SMALL_PIC_WIDTH_HEIGHT+col] = WHITE;
                }
            )
        }
    );
    res
}

pub fn test(file_name: &str) {

    let bytes = get_zlib_file_bytes(file_name).unwrap();

    /// 展示缩略图
    // let bits = get_thumb_stream(&bytes).unwrap();
    // let image_v = bits.iter().map(|x| if *x == 0 { WHITE } else { BLACK }).collect();
    // let image_width = THUMB_STREAM_WIDTH_HEIGHT as u32;
    let begin = Instant::now();
    let image_v = get_full_stream(&bytes);
    println!("{:?}",begin.elapsed().as_millis());
    let image_width = (THUMB_WIDTH_HEIGHT * SMALL_PIC_WIDTH_HEIGHT) as u32;

    /// 展示小图
    // let image_v = get_small_stream_with_pos(&bytes, 2);
    // let image_width =  SMALL_STREAM_WIDTH_HEIGHT as u32;
    // println!("{:?}", image_v.len());

    // println!("size:{:?}", image_v.iter().enumerate()
    //     .filter(|(x, y)| **y == WHITE)
    //     .map(|(x, y)| x)
    //     .collect::<Vec<usize>>()
    // );

    image_show(image_width, image_width, image_v);
    sleep(Duration::from_secs(5));

    // 保存到文件中
    // let mut image = image::GrayImage::new(image_width, image_width);
    // // x width, y height
    // for i in 0 .. SMALL_STREAM_WIDTH_HEIGHT * THUMB_STREAM_WIDTH_HEIGHT {
    //     for j in 0..SMALL_STREAM_WIDTH_HEIGHT * THUMB_STREAM_WIDTH_HEIGHT {
    //         image.put_pixel(i as u32, j as u32,
    //                         Luma::from([*image_v.get(j*image_width as usize+i).unwrap()]))
    //     }
    // }
    // image.save("c.jpg").unwrap();

    //指定质量保存
    // let out_file = File::create("d.jpg").unwrap();
    // let mut jpg_encode = image::codecs::jpeg::JpegEncoder::new_with_quality(out_file, 100);
    // jpg_encode.encode(
    //     image_v.as_slice(),
    //     image_width,
    //     image_width,
    //     ExtendedColorType::L8
    // ).unwrap();

}





