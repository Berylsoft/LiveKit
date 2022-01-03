# LiveKit Alpha阶段功能说明

**LiveKit是一个实验性的项目，尚处在活跃开发阶段，大部分功能和模块都不稳定，且尚未准备好正式发布。** 这也是为什么需要独立出一个功能说明，以用于小范围内的测试，而不是直接在rustdoc或者README中交代这些内容。

现阶段尽量不要将LiveKit用于生产环境及任何关键场合，较稳定的部分（以下会详细说明）可以酌情使用。功能及接口（包括库的API及主程序和工具的命令行用法、配置文件、输出数据的格式等）随时可能发生变动，请谨慎依赖于现有版本。**使用时有任何问题可以添加反馈讨论群（QQ群751585393），或者也可以随时通过B站私信或者邮箱联系我。**

构建方式：安装好rustup后在仓库根目录运行`cargo build --release`即可，可执行文件会出现在/target/release/位置，运行只需要可执行文件本身

对于Windows用户可以直接下载使用我构建好的：[e8bad13](https://berylsoft-assert-1307736292.file.myqcloud.com/livekit-alpha/release-v03-e8bad13-x86_64-pc-windows-msvc.7z)

项目结构见README，以下列出各个模块的功能介绍、开发状态和使用说明。可执行文件在前，库在后，**但有很多注意事项在库的介绍中，请测试参与者务必阅读完整个文件。**

请库调用者（短期内不会有吧，应该绝大部分是用可执行文件）注意：feature控制目前存在一些问题，而且很玄学，不好排查解决。不过目前按照所需功能正常启用相应feature就行，至少能用肯定是能用，冗余问题慢慢解决（或者我其实觉得意义不大，又不是很底层的库，之后把所有feature删了也不是不可能）。

## livekit (bin: livekit)

主程序。

| option/flag | description |
|-------------|-------------|
| `-c, --config-path` | 配置文件路径 |

TOML配置文件示例：

```toml
[[group]]
rooms = [ # 完整房间号或者短号
    24393,
    -399854, # 前有负号代表此房间不开启（语义上相当于录播姬不监控）
    876396,
    -14327465,
]
[group.config]
# 信息流原始数据存储
storage.path = "C:\\swap\\livekit-testing\\long2" # 目前必填
# 信息流解析后数据记录，输出schema参考 feed/schema.rs，之后可能会想办法生成一个json schema
dump.path = "C:\\swap\\livekit-testing\\dump" # 目前必填
dump.debug = false # false输出流式json，同时不输出未实现和忽略的事件；true输出rust debug
# http配置
# http.access = { uid = 1130367996, key = "********************************", csrf = "********************************" } # 账号登录信息
# http.proxy = "https://example.com" # api.live.bilibili.com的代理
# 录制目前还是残废，连自动开始录制都没有
# record.mode = "FlvRaw"
# record.path = "C:\\swap\\livekit-testing\\rec"
# record.name_template = "{roomid}-{date}-{time}{ms}-{title}" # 参考下方模板列表
# record.fragment = { type = "ByTime", per_min = 60 } # 目前不起作用
# record.qn = [10000] # 目前不起作用，目前逻辑是有20000就录20000，没有就录10000
```

LiveKit主程序采用了一种叫房间组的策略，**同一个主程序实例可以运行多个房间组，每个房间组每一个房间的设置都是相同的，不同房间组的设置都是不同的**。以上示例是一个房间组的配置，每个配置文件可以有多个房间组，将以上示例复制多份即可。

视频流录制文件名模板（均为录制开始时）：

| key | value |
|-----|-------|
| date    | 日期（YYmmdd） |
| time    | 时间（HHMMSS） |
| ms      | 毫秒数 |
| iso8601 | 时间iso8601格式，精确到毫秒 |
| ts      | 时间戳，精确到毫秒 |
| random  | 00-99的随机数 |
| roomid  | **完整**房间号 |
| title   | 标题 |
| name    | 主播用户名 |
| parea   | 父分区 |
| area    | 分区 |

目前日志使用env_logger输出到stderr，日志级别由环境变量`RUST_LOG`指定，建议设为info级别。

## livekit-feedrec (bin: feedrec)

信息流原始数据记录工具。这个工具和它用到的功能已经在我的生产环境测试了两个月。

| option/flag | description |
|-------------|-------------|
| `-r, --roomid-list` | 要记录的多个**完整**房间号，以**半角逗号**隔开 |
| `-s, --storage-path` | 存储文件夹的路径 |
| `-l, --log-path` | 记录日志的路径（日志记录使用log4rs，默认为info级别，不受环境变量影响，不输出到stdout或者stderr） |
| `--log-debug` | 输出debug级别日志<br>（文件会很大，除非检查问题否则一般不需要开启，不过我这里的生产环境常开） |

## livekit-feed-dump (bin: feed-dump)

信息流原始数据存储解析工具。注意解析生成的是原始json数据，而非主程序输出的重整格式。

| option/flag | description |
|-------------|-------------|
| `-r, --roomid` | 要解析的 |
| `-s, --storage-path` | 要解析的存储文件夹的路径 |
| `-o, --export-path` | 输出文件的路径 |
| `--rocks-ver` | （可选）如果解析的是rocksdb格式存储，则要解析的存储的rocksdb的版本字符串<br>（早期使用rocksdb格式存储的版本从未在除了我这里的地方使用，所以忽略即可） |

未来会加入可选的输出重整格式，以及更细致的过滤器（如要解析的时间段）。

## livekit-interactor (bin: livekit-interactor)

命令行直播互动工具。目前仅支持发弹幕。

| option/flag | description |
|-------------|-------------|
| `-a, --access` | 账号登陆信息文件路径，写为json，[格式参考](/api/client.rs#L42)，举例：<br>`{"uid": 1130367996, "key": "********************************", "csrf": "********************************"}` |
| `-p, --payload` | 要执行的操作，写为json，[格式参考](/bin/interactor/main.rs#L13)，举例：<br>`{"type": "Danmaku", "data": {"roomid": 24393, "msg": "test", "emoji": false}}`<br>注意其中房间号也为**完整**房间号 |

## livekit-api

HTTP请求与REST API封装库。包含一个便于API调用的http client封装及各类直播相关的API定义及调用实现。

| module | description |
|--------|-------------|
| client | http client封装 |
| info | 房间基本信息相关 |
| feed | 信息流连接 |
| stream | 视频流获取 |
| interact | 互动相关 |

稳定性：已有代码和包括的API基本稳定（也没啥可不稳定的），除http client的access（账号登录信息管理）部分的解析原始cookie函数未经测试外。未来会不断增加支持的API；可能会改变调用方式；可能会换用更底层的hyper库。

## livekit-feed

信息流协议及数据解析支持库，也是目前打磨最细致的部分。

*信息流即常说的“弹幕服务器”，以tcp或者websocket方式推送弹幕、礼物等互动信息与直播间各类信息更新。

| module | description |
|--------|-------------|
| config | 配置常量 |
| util | 时间戳与json解析工具函数 |
| package | 二进制包解析和构造 |
| stream | 连接与接收流实现，可选tcp/websocket |
| schema | 包内容json解析和重整（输出人和机器都更易读的数据） |

稳定性：已有代码基本稳定，经过生产环境测试。schema会不断增加支持的信息类型，目前弹幕、进场、礼物、SC、上舰、上下播、房间信息变化、关注数粉丝牌数这些基本的信息都可以解析。对于SC发送者的粉丝牌颜色解析可能存在问题。

## livekit-feed-storage

信息流原始数据存储相关封装库。

| module | description |
|--------|-------------|
| (root) | 封装好的数据库操作函数 |
| rec | 仅记录信息流原始数据的线程，主要用于feedrec |
| sled | 重导出的sled库，请库调用者不要直接添加sled依赖，而是使用该重导出 |

稳定性：已有代码基本稳定（也没啥可不稳定的）。

## 关于信息流原始数据存储

livekit的主程序和feedrec工具（之后会介绍）会记录所有开启的直播间的信息流原始数据。原始数据即未经任何解包和解析处理的，从tcp或者websocket流上收到的原始二进制包。**我认为要做弹幕记录，保存原始数据是必要的，因为解析实际上是一个丢弃部分信息的有损过程，而且数据的格式随时可能发生变化，解析很可能处理不到。**

本模块采用sled这个纯rust实现的现代键值存储引擎来存储原始数据，目前版本锁定为0.34.7。sled的存储是一个文件夹，有以下内容：db是数据，conf是数据库配置，snap（大概）是上次的memcache索引，blobs文件夹（大概）是存放较大的value（在我们的场景下一般为空）。主程序每个房间组（之后会解释）一个存储，feedrec所有房间一个存储。使用**完整房间号的字符串**作为键命名空间，**以毫秒为单位的64位unix timestamp**作为键，包原始二进制数据作为值。需要特别注意的是sled的memcache的会占用一部分内存，目前的设置是最大16MB，这会构成可执行文件运行时内存占用的一部分。

## livekit-feed-client

信息流client封装库。

| module | description |
|--------|-------------|
| thread | 当前的基于线程的client实现 |

稳定性：已有代码基本稳定（也没啥可不稳定的）。目前的线程实现（其实就是加上了发解析后event的channel的feed-storage rec线程）只是临时解决方案，之后会重构为actor。

## livekit-stream-get

直播流拉取与记录库。

| module | description |
|--------|-------------|
| config | 配置常量 |
| url | 解析并构造url |
| flv | flv录制实现 |

稳定性：直播流信息解析基本稳定。**录制目前还是残废，连自动开始录制都没有**。录制线程实现只是临时解决方案，之后会重构为actor。

## livekit-stream-parse

直播流解析与修复库。

规划中，暂时为空。

## livekit (lib)

主程序与所有功能的封装库。

| module | description |
|--------|-------------|
| config | 配置文件的schema |
| room | 目前所有功能的封装形式（Room结构体） |

稳定性：目前的Room结构体只是临时解决方案，之后会重构为actor。
