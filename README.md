# calculator
## 安装rust
参照<https://www.rust-lang.org/zh-CN/learn/get-started>
安装rust。首次安装默认安装最新版本的rust即可
## 升级rust
运行`rustup update`
## 验证安装成功
执行`cargo --version`
## 编译项目
在README.md同级目录执行 `sh compile.sh`
### 编译后的二进制文件地址
./target/release/calculate
## 启动项目
./calculate
## 启动的项目端口
8090
## 健康检查
`curl http://127.0.0.1:8090/hello`
## 日志位置
./logs
### 日志轮转
1. 保留30天的日志，1天前日志压缩
2. 每天轮转一次日志或500M轮转一次
