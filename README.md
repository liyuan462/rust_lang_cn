# rust_lang_cn
**China Community for Rust lang**

### 如何运行
如果要在本地搭建[Rust China](http://rust-lang-cn.org/)测试环境，请参见以下步骤：

1. clone本仓库到本地
2. 初始化MySQL数据库，创建rust_lang_cn数据库，创建各数据表，建表语句见tables.sql
3. 拷贝config-sample.toml至config.toml，修改其中的数据库配置
4. 拷贝log4rs-sample.yaml至log4rs.yaml，可以不用修改
5. 配置并运行静态文件服务器，服务static文件下的静态文件内容，可用Nginx或者简单地用```python -m SimpleHTTPServer```亦或者```php -S 0.0.0.0:8000```，并设置好config.toml中的static_path，这个路径应该和你启动的静态资源服务器的资源所在路径相符合。
6. 编译，执行命令```cargo build --release```即可
7. 设置好config.toml中的app_path为：```"http:://localhost:3000"```，然后运行```./target/release/rust_lang_cn```
8. 访问[http://localhost:3000](http://localhost:3000)

### 如何修改css
* css采用sass来编写，产生好的css文件在static中：```static/css/base.css```
* sass源码在```src/sass```中，主文件为```src/sass/base.scss```，其中集成了Bootstrap的sass源码，修改或添加样式只要修改```src/sass/base.scss```，然后用sass编译输出到```static/css/base.css```，具体命令如下：

**你可以手动编译**
```
cd src/sass
sass base.scss ../../static/css/base.css
```

**使用 gulp 监听自动编译**
```
npm i
gulp
```

### 目前已有功能

* 注册
* 登录
* 发表话题
* 编辑话题
* 回帖
* 个人中心
* RSS
* 置顶，加精

### 如何参与

* 如果你有问题，请提[issue](https://github.com/rust-cn/rust_lang_cn/issues)或者在[Rust China](http://rust-lang-cn.org)的[站务](http://rust-lang-cn.org/category/6)版块发帖。
* 如果你想贡献代码，请提[pull request](https://github.com/rust-cn/rust_lang_cn/pulls)
* 如果你想成为核心开发团队的一员，请[联系我们](mailto:admin@rust-lang-cn.org)
* 还有，别忘了加入我们的[交流频道](https://rust-cn.pubu.im/reg/prmkl2w7n2n9fky)

### 常见问题

* mac上编译找不到openssl，参见这个[issue](https://github.com/rust-cn/rust_lang_cn/issues/7)
