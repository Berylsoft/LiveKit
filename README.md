# [WIP] Berylsoft LiveKit

An experimental Bilibili Live support library with utilities. Written in Rust, design to be fast, correct, robust and portable.

| crate | type | description |
|-------|------|-------------|
| livekit-api | lib | HTTP请求与REST API封装 |
| livekit-feed | lib | 信息流协议及数据解析 |
| livekit-feed-client | lib | 信息流封装 |
| livekit-stream | lib | 直播流拉取与记录 |
| livekit-flv | lib | FLV直播流解析与修复 |
| livekit-hls | lib | HLS直播流解析与修复 |
| livekit | lib+bin | 所有功能封装，主程序 |
| livekit-feedrec | bin | 信息流记录程序 |
| livekit-feed-dump | bin | 信息流原始数据存储解析程序 |
| livekit-interactor | bin | 命令行直播互动工具 |
