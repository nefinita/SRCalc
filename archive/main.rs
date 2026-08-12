use std::iter::Inspect;

fn main() {
    println!("欢迎使用星铁排轴计算小助手");
    println!("如果输错了直接关掉重开");
    println!("输入数字即可使用指定工具");
    println!("输入q即可退出");
    println!("1 四角色排轴计算器   2 角色配装优化器");

    

}

pub fn clear_screen() {
    let _result = if cfg!(target_os = "windows") {
        std::process::Command::new("cls")
            .status()
            .unwrap();
    } else {
        // "clear" or "tput reset"
        std::process::Command::new("clear")
            .status()
            .unwrap();
    };
    
}