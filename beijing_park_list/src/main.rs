use regex::Regex;

const FIELD_SEPERATOR: &str = "\t";

// 读取北京公园列表，转化为可读取的文本/csv
fn main() {
    // 读取文件内容，pdf下载地址：https://yllhj.beijing.gov.cn/ggfw/bjsggml/bjsgymlzb/202408/P020240822512698668915.pdf
    let bytes = std::fs::read("./pdf/beijing_park_list.pdf").unwrap();
    let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();

    // 一行中的换行，全部换成/，即非连续的\n替换成斜杠
    let line_slash_n = Regex::new(r"([^\n])\n([^\n])").unwrap();
    let q = line_slash_n.replace_all(out.as_str(), "$1/$2");

    let begin_with_num = Regex::new(r"^[0-9]+").unwrap();
    let phone_num = Regex::new(r"[-0-9]{5,}").unwrap();

    // 表头
    let header = q.split("\n").find(|x| x.contains("序号")).unwrap()
        .split(" ").collect::<Vec<&str>>().join(FIELD_SEPERATOR);

    // 表内容
    let b = q.split("\n")
        .filter(|x| begin_with_num.is_match(x)) // 只保留以数字序列开头的行
        .map(|x| x.replace("  ", " ")) // 将多个空格替换成一个空格
        .map(|x| { // 每一行格式正规化，以\t分格
            let fields = x.split(" ").collect::<Vec<&str>>();
            let mut ret = fields[..7].join(FIELD_SEPERATOR);
            let mut now_index = 7;
            // 有一些地址会有空格，所以通过后续是否是电话号码来差别第7个是否是地址的一部分，还是电话的一部分
            if !phone_num.is_match(fields[7]){
                ret = ret + fields[7];
                now_index+=1;
            }
            // 电话也有可能有空格，所以规则是除了最后一列，其它的都是电话号码
            let phone_filed_end_index =  fields[now_index..].len() - 1+now_index;
            ret + FIELD_SEPERATOR +
                fields[now_index..phone_filed_end_index].join("").as_str()
                +FIELD_SEPERATOR+fields[phone_filed_end_index..].join("").as_str()
        })
        .collect::<Vec<String>>().join("\n");

    // 写入文件，导出的飞书地址：https://e22mq1224o.feishu.cn/sheets/GWihsVTDFhHuD6tWKzIcMAB2nld
    std::fs::write("./output/beijing_park_list.list", header+"\n"+b.as_str()).unwrap();
}
