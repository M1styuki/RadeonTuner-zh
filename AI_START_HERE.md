# RadeonTuner 简体中文版：项目记录

## 当前交付与授权
- 私有源码仓库：https://github.com/M1styuki/RadeonTuner-zh 。用户明确授权上传源码、云端编译、下载覆盖 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe。
- 2026-11-06 用户要求本轮完成后关机；若账户任一额度剩余1%则立即记录、停手关机。此授权覆盖旧记录中的“不再关机”。关机前不等待睡眠中的用户验收。
- 当前EXE已覆盖并校验：0.70.0.0，3136000字节，SHA256 C1C12BE27E3A2FE29D8ECF0A8796492C9CCB4D4E080F2641729363C327923BBB。
- 正式core位于 C:\Users\78539\AppData\Local\RadeonTuner-zh-CN\App\RadeonTuner.exe，SHA256 6C667F64BD03A734F124F62441AE4B645CD62689A6A818B7200ABEF7A07CA604。
- 设置位于同目录 Settings\Settings.json，保留用户设置、运行文件和共享运行库。当前项目重新开启开发，源码/DSH历史保留至用户明确满意并结束项目；只清理本轮可再下载临时文件。

## 已确认行为与界面
- 上游 dumbie/RadeonTuner 0.70，已合并 dfade6b（含c120f54a）；保留许可证，仅私用。
- 仅简体中文、深色模式、原生中文深色右键菜单。保留原驱动功能/设置语义，隐藏托盘为新增设置。
- 无卡片、小标题、分隔线；左说明及?/!，右控件共用右边缘。数值/下拉框160x32 DIP，数字居中，滑块等长。监控13项5列3行；风扇图完整外框/细网格，五点成对输入，最低最高转速同行滑块。
- 真实条件项折叠、页面/下拉框短过渡，44x32开关从当前圆点位置即时反向，保留原生键盘/拖动/双击输入。不添加驱动失败、迁移或其他模拟。
- 图标由原256PNG按像素提取白色主体与抗锯齿边缘，背景透明，原实心白色保持，心形内部alpha255；三个ICO均含256/64/48/32/24/16。两次生成图像质量失败未纳入交付。

## 本轮内存结构
- App持有后台配置任务，UI为弱会话；关闭到托盘120ms后停UI计时器/动画，快照草稿，断开Content并Close DesktopWindowXamlSource，释放MainPage。再次EXE创建页面恢复草稿，不重复驱动初始化/默认写入/后台任务。
- WindowsXamlManager/Dispatcher保留到线程退出（微软不支持同线程Close后重建manager）。不能承诺框架全部卸载或零后台占用。
- 面板不存在时暂停性能、图形状态及所选设备采集；配置保持/应用配置/Gamma/Eyefinity后台继续。后台不得持有页面裸this。
- 独立审查发现并已修复：5处picker等待旧会话取消、暂态overlay不恢复、timer单次Tick注册并清理、采样保留后台并检查expectedSession、退休页不新建通知。构造采用make_self，所有8处get_self先转换投影接口。独立审查已复核这些修复。

## 实际检查与限制
- 云端只编译/打包，不运行应用或驱动测试。本机未重新安装VS/SDK。
- run42060814225 的Release编译/正式打包/正式工件上传成功，artifact13432567578；ZIP SHA256 44F1D3CD7574A26A155FFE2F8AA09C2F4F2082716BFC6C11B9019C94F39CFD53，EXE与打包报告一致。
- 同run的UiValidation编译失败：UiValidationScenario.h:1352少winrt限定，已一行修复，需后续云编译确认。正式版不编译该隔离检查代码。
- 用户明确要求下载后本机实测，已覆盖此前“禁止普通启动core”的早期限制。实际正常退出旧版、启动新版，关闭/唤回两次均正常，同一PID1772，无重复进程，原设置页面恢复、白图标正常。未改动任何调校值，也未模拟驱动失败/迁移。
- 专用工作集（MiB，WorkingSetPrivate）旧版PID8696打开89.84、关闭90.04；新版PID1772首次隐藏46.16、打开94.79、关闭54.41、再次打开98.77、再次关闭57.87。新版关闭确实回落，旧版关闭无回落。不同进程/运行时长，仅为此次本机样本，不能宣称全面性能/驱动功能验收或固定降幅。
- 原始数据在 .build-work/evidence/*memory.json。进程WS/PrivateBytes也已记录，不能与专用工作集混用。
- 本轮未执行隔离生命周期运行检查；正式开关面板结果不能证明所有后台驱动功能长期正常。前一轮开关动画/布局验收记录见Git历史。

## 开发与恢复
- 主入口 RadeonTuner/MainPage.xaml、MainPage.cpp、App.xaml、App.cpp；构建 .github/workflows/build.yml，windows-2025-vs2026；单EXE tools/Portable；静态 tools/verify-localization.ps1、tools/verify-ui-layout.ps1。
- 活跃DSH session-c605187d-a18a-4ef4-8d4a-353f838e7a13，当前无任务运行。保留该会话及源码至明确项目结束；旧会话已在上轮明确收尾时删除，不得猜测续接。
- 本地main a451d89、远端main a31d1c4abebdba9c67bba70d466422256370b50e，同树63dae5a3c7018ab39a1b9c140347ae65956a4fd8。普通Git到github.com失败，插件Git对象API可用；本地origin/main仍旧489fa50，勿盲推。
- API发布逐个blob及最终tree核对本地Git，expected_sha非强制更新main。下一次增量以本地a451d89、远端a31d1c4a为基线；.build-work/Prepare-GitHubUpload.ps1辅助生成原始blob的base64清单。API提交元数据导致commitID不同，但文件树一致。
- 上轮开发环境已官方卸载：VSBuildTools18.11.3实例98facc78/SDK26100/VSInstaller；本轮无本机编译工具安装。保留原.NET6/8/10、Desktop8/10、VC++v14 x64/x86共享运行库及原DOTNET_TieredCompilation=0。
- 上轮清理所有会话outputs已完成，禁止重复跨项目清理。当前只允许清理可明确归属的下载ZIP、临时解压/图标预览，不删除设置或其他项目。

## 2026-11-06 关机前交接
- 最新本地3f52b830、远端4de9deacde3bbf43d7d5dd40614bd9e928709333，同树78c5afd68e36b1c5642330b795362833af087b84。该提交仅修复隔离检查类型限定并更新记录，未改正式版源码。
- 新云编译run42065045829仍在执行，仅编译打包；关机不会停止GitHub任务。正式交付已完成本机检查和覆盖，不等待未交付的隔离检查版。未宣称该修复已编译通过。
- 已清理本轮下载ZIP/解压正式工件及icon-review临时预览，保留正式EXE、运行文件/设置、源仓库/DSH恢复历史，以及evidence内存JSON和打包报告。本轮未安装本机开发环境。移除工件可从GitHub重新下载，图标预览可由原图重建。
- 下一步按最新用户明确授权执行正常关机，不强制终止其他应用；若系统阻止关机，不能声称机器已经关闭。
