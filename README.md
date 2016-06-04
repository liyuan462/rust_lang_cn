# rust_lang_cn
**China Community for Rust lang**

### 如何运行
如果要在本地搭建[Rust China](http://rust-lang-cn.org/)测试环境，请参见以下步骤：

1. clone本仓库到本地
2. 初始化MySQL数据库，创建rust_lang_cn数据库，创建各数据表，建表语句见tables.sql
3. 拷贝config-sample.toml至config.toml，修改其中的数据库配置
4. 拷贝log4rs-sample.yaml至log4rs.toml，可以不用修改
5. 配置并运行静态文件服务器，服务static文件下的静态文件内容，可用Nginx或者简单地用```python -m SimpleHTTPServer```，并设置好config.toml中的static_path
6. 编译，执行命令```cargo build --release```即可
7. 运行```./target/release/rust_lang_cn```并设置好config.toml中的app_path为：```"http:://localhost:3000"```
8. 访问http://localhost:3000

### 如何修改css
* css采用sass来编写，产生好的css文件在static中：```static/css/base.css```
* sass源码在```src/sass```中，主文件为```src/sass/base.scss```，其中集成了Bootstrap的sass源码，修改或添加样式只要修改```src/sass/base.scss```，然后用sass编译输出到```static/css/base.css```即可

### 目前已有功能

* 注册
* 登录
* 发表话题
* 编辑话题
* 回帖
* 个人中心

### TODO

* 邮箱验证


