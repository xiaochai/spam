# 用于抓取flightradar24网站的KML文件

## 编译

```shell
cargo build --bin get_kml --release
```

## 运行

```shell
// 批量抓取
./target/release/get_kml --file ./src/config/travel.list
// 单个抓取
./target/release/get_kml CA1223 2024-10-09
```

抓取的文件位于./data/kml/目录中

配置文件位于./src/config/application.yml中，需要填入网站的用户名和密码，并且确保是会员