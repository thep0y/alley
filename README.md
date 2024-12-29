<p align="center"><img height="128" width="128" src="./src-tauri/icons/icon.png" /></p>

# FLUXY

新版正在龟速开发中，比重写一个项目可能还要慢，加上我没有多少空闲时间，不知道猴年马月能写完了。

---

新版使用 udp 组播搜索发现、 tcp 传输数据的方案重构，需要在一个局域网中的两台电脑边开发边调试，短期内没有这个条件，所以新版暂时搁置，时间充裕了才能继续开发。

## 开发

### Android

查看日志：

```bash
./adb logcat --format=raw -s RustStdoutStderr
```