# RadeonTuner 简体中文版：项目记录

## 当前目标与交付
- 私有源仓库 https://github.com/M1styuki/RadeonTuner-zh ，用户授权源码上传、仅云端编译、下载覆盖 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe；最终仅此一个EXE，首次运行可释放必要运行文件与设置。
- 2026-11-06 新要求：4K+150% DPI托盘图标模糊；后台约50MB仍过高。用户明确批准“轻量原生后台＋按需独立面板进程”，保留超频、降压、功耗、风扇及原功能。
- 最新用户明确“不用实测，我自己能看”：禁止自动启动/操作应用、GUI/驱动/内存实测和模拟；只相关代码检查、云编译、下载覆盖，由用户验收。
- 前轮关机指令已成功下达，现用户已恢复使用电脑，本轮不自动关机。源码/活跃DSH历史保留至用户明确满意并结束项目，不能依据旧收尾授权删除当前开发项目。
- 当前交付仍是前轮EXE：3136000字节，SHA256 C1C12BE27E3A2FE29D8ECF0A8796492C9CCB4D4E080F2641729363C327923BBB。新候选尚未云编译，不得当作已交付。
- 运行根 C:\Users\78539\AppData\Local\RadeonTuner-zh-CN\App；Settings\Settings.json与Profiles保留，不读写用户配置做验证。

## 保留的产品行为
- 基于 dumbie/RadeonTuner 0.70，已合并dfade6b（含c120f54a），保留许可证，仅私用。
- 仅简体中文/深色界面/原生中文深色右键菜单。无卡片、小标题、分隔线；左说明与?/!，右控件对齐，160x32输入/下拉框、数值居中、等长滑块。监控13项5列3行，风扇图完整边框细网格，成对数值及最低最高滑块。
- 条件项折叠和短过渡，44x32开关即时反向，保留原生键盘/拖动/双击。隐藏托盘设置保留；不添加驱动兼容修补、失败/迁移模拟或新功能。
- 白色透明图标来自原图精确像素去底，三个ICO含256/64/48/32/24/16，未重绘。新托盘按任务栏GetDpiForWindow/GetSystemMetricsForDpi取尺寸，144DPI为24px；仅尺寸变化刷新，主题/显隐/TaskbarCreated仍显式刷新，HICON独立所有权。

## 新进程架构（候选已完成，待编译）
- 同一core默认角色为原生host，保留RadeonTunerMainWindow控制入口，不创建MainPage/WindowsXamlManager/WinUI；独立-ui-worker角色使用RadeonTunerUiWorkerWindow，至多一个。外部仍交付一个EXE。
- host headless ADL/配置初始化，默认registry/AdlAppSyncAll每host一次；worker各自建立ADL上下文，不能跨进程传ADL句柄，不重复全局默认写入/同步。性能采集仅worker有，后台保持/应用匹配/Gamma/Eyefinity保留。
- 启worker前PauseProfileLoop等待host当前批次完成；worker期间host暂停，关闭worker前亦暂停其批次。后台与worker不会同时轮询写驱动；worker结束后host重读Settings/Profiles并恢复实际生效档，再恢复后台。
- AVSettingsJson构造缓存整文件，交接重新构造；HideTrayIcon/ThemeIconMono/ThemeStyle由worker通知所属host刷新，不再200ms整文件读盘。
- 匿名继承双向pipe、固定长度头、8MB上限、STARTUPINFOEX handle-list，host内存保存模型/选择/草稿，无永久会话文件或公开配置格式变更。完整内部模型保留Current/Default/范围/Step/Support/UsingProfile等，optional字符串null保持。
- 每次开worker在暂停后捕获最新runtime flags，合并到保留草稿，按DeviceId+Application对齐UI UsingProfile；后台实际生效cache标志与UI选中草稿分离，不能ReloadProfiles冒称Global已生效。设备身份失配不套错snapshot。
- 正反消息使用所属host HWND/PID及worker PID校验；closing先通知，快召回排队。正常完整退出异步握手继续读pipe；worker等待UI工作及对应分辨率回滚后捕获/退出。Custom Stop已改IAsyncAction并await真实删除，无固定Sleep充当完成。
- host在worker未建窗时记完整退出并在轮询重试，整套退出前不再恢复后台批次。Job kill-on-close仅防孤儿，SILENT_BREAKAWAY_OK保留Updater/外部工具；更新成功走整套退出且co_return；初始化失败有原错误UI入口，不启无效ADL循环。

## 代码检查与限制
- DSH同会话实施DPI、A headless接口、B进程/IPC及两轮定向修复。Codex补namespace/投影接口/host角色标志，以及最后3项暂停批次、等待Custom删除、建窗后重试退出。
- 独立审查已确认headless路径无XAML激活、批次暂停无循环等待；已发现并修复模型元数据/cache flags陈旧、退出堵pipe/强杀回滚、跨副本召回、设置轮询、optional null和更新器退出问题。最后限定复核未发现残留或明显类型错误，可以云编译。
- 实际静态：汉化12项、布局7页104行、diff--check通过。没有运行程序或采集本轮内存，不承诺具体降幅，也不把编译等同于真实超频/Gamma/Eyefinity验收。
- UiValidation保留原隔离同进程检查入口，只构建不运行，不能证明正式跨进程生命周期。前轮run42065045829已核对success，本轮需新云工件。

## 开发/发布/清理
- 活跃DSH session-c605187d-a18a-4ef4-8d4a-353f838e7a13，保留会话及源码直至明确项目结束；当前无DSH任务运行。
- 入口 AppWinMain.cpp、App.cpp、MainPage.cpp；IPC RadeonTunerIpc.h；backend RadeonTunerBackend.h；构建 .github/workflows/build.yml/windows-2025-vs2026，正式包先上传，后编译UiValidation，不在runner运行应用。
- 本轮开始本地3e516a8f77d7d81a1e3db648774e36b012556146、远端6a2b253b7d71fb6408969b435b211575ac994f61，同树5499310e7384c012268e5db184be3e5c4547a468。普通Git网络曾失败，API对象发布逐blob/tree核对，expected_sha非强制更新；不要盲推旧origin/main。
- .build-work/Prepare-GitHubUpload.ps1及source-upload-manifest.json保存最新本地/远端基线与完整树，待本轮发布后记录映射。下载放本任务.build-work，校验ZIP和EXE报告，再覆盖指定交付；清理可再下载ZIP/解压副本，保留发布证据/恢复文件。
- 本轮无本机编译工具安装。上轮VSBuildTools18.11.3实例98facc78/SDK26100/Installer已卸载；原.NET6/8/10、Desktop8/10、VC++v14 x64/x86及DOTNET_TieredCompilation=0保留。禁止重复跨会话outputs清理，禁止动其他项目/设置/共享运行库。
