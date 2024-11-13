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
## 性能指标
2c 2G内存下 cpu 100% 内存只使用了40Mi

平均时延：395.98ms

每秒处理请求数：1006.14

最差响应时延：700.99ms


### ab测试结果
```
ab -n 100000 -c 100 -p "post_data.json" -T "application/json" -H "Content-Type: application/json" -H "Cache-Control: no-cache" "http://100.122.108.1:8090/v1/calculate_rating"
This is ApacheBench, Version 2.3 <$Revision: 1913912 $>
Copyright 1996 Adam Twiss, Zeus Technology Ltd, http://www.zeustech.net/
Licensed to The Apache Software Foundation, http://www.apache.org/

Benchmarking 100.122.108.1 (be patient)
Completed 10000 requests
Completed 20000 requests
Completed 30000 requests
Completed 40000 requests
Completed 50000 requests
Completed 60000 requests
Completed 70000 requests
Completed 80000 requests
Completed 90000 requests
Completed 100000 requests
Finished 100000 requests


Server Software:
Server Hostname:        100.122.108.1
Server Port:            8090

Document Path:          /v1/calculate_rating
Document Length:        108 bytes

Concurrency Level:      100
Time taken for tests:   101.859 seconds
Complete requests:      100000
Failed requests:        2
   (Connect: 0, Receive: 0, Length: 2, Exceptions: 0)
Total transferred:      21699998 bytes
Total body sent:        32600000
HTML transferred:       10799998 bytes
Requests per second:    981.75 [#/sec] (mean)
Time per request:       101.859 [ms] (mean)
Time per request:       1.019 [ms] (mean, across all concurrent requests)
Transfer rate:          208.05 [Kbytes/sec] received
                        312.55 kb/s sent
                        520.60 kb/s total

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       4
Processing:     2  102  72.4     98     310
Waiting:        2  102  72.4     98     310
Total:          2  102  72.4     99     310
```
* post_data.json文件
```
{"clients":[{"uid":"1","hands":["As","Ad"]},{"uid":"2","hands":["2s","Ts"]},{"uid":"3","hands":["3s","Js"]}]}
```
### wrk测试结果
```
# wrk -t20 -c400 -d60s -v -s post.lua "http://100.122.108.1:8090/v1/calculate_rating"
wrk 4.2.0 [epoll] Copyright (C) 2012 Will Glozer
Running 1m test @ http://100.122.108.1:8090/v1/calculate_rating
  20 threads and 400 connections



  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   395.98ms   23.22ms 700.99ms   98.49%
    Req/Sec    56.67     38.85   595.00     72.09%
  60462 requests in 1.00m, 12.51MB read
Requests/sec:   1006.14
Transfer/sec:    213.22KB
```

* post.lua

```
wrk.method = "POST"
wrk.body   = "{\"clients\":[{\"uid\":\"1\",\"hands\":[\"As\",\"Ad\"]},{\"uid\":\"2\",\"hands\":[\"2s\",\"Ts\"]},{\"uid\":\"3\",\"hands\":[\"3s\",\"Js\"]}]}"
wrk.headers["Content-Type"] = "application/json"
wrk.headers["Cache-Control"] = "no-cache"
```
