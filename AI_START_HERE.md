# RadeonTuner 简体中文版：项目记录

## 当前目标与交付
- 私有源仓库 https://github.com/M1styuki/RadeonTuner-zh ，用户授权源码上传、仅云端编译、下载覆盖 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe；最终仅此一个EXE，首次运行可释放必要运行文件与设置。
- 2026-11-06 新要求：4K+150% DPI托盘图标模糊；后台约50MB仍过高。用户明确批准“轻量原生后台＋按需独立面板进程”，保留超频、降压、功耗、风扇及原功能。
- 最新用户明确“不用实测，我自己能看”：禁止自动启动/操作应用、GUI/驱动/内存实测和模拟；只相关代码检查、云编译、下载覆盖，由用户验收。
- 前轮关机指令已成功下达，现用户已恢复使用电脑，本轮不自动关机。源码/活跃DSH历史保留至用户明确满意并结束项目，不能依据旧收尾授权删除当前开发项目。
- 最新EXE已覆盖并校验：3183616字节，SHA256 B093E80771CCD445F561A790EF9B46EFB2CC0E00489C3DC787589D81762DCCE1；run42215572575的Release与UiValidation均编译/打包成功，未运行应用。
- 运行根 C:\Users\78539\AppData\Local\RadeonTuner-zh-CN\App；Settings\Settings.json与Profiles保留，不读写用户配置做验证。

## 保留的产品行为
- 基于 dumbie/RadeonTuner 0.70，已合并dfade6b（含c120f54a），保留许可证，仅私用。
- 仅简体中文/深色界面/原生中文深色右键菜单。无卡片、小标题、分隔线；左说明与?/!，右控件对齐，160x32输入/下拉框、数值居中、等长滑块。监控13项5列3行，风扇图完整边框细网格，成对数值及最低最高滑块。
- 条件项折叠和短过渡，44x32开关即时反向，保留原生键盘/拖动/双击。隐藏托盘设置保留；不添加驱动兼容修补、失败/迁移模拟或新功能。
- 白色透明图标来自原图精确像素去底，三个ICO含256/64/48/32/24/16，未重绘。新托盘按任务栏GetDpiForWindow/GetSystemMetricsForDpi取尺寸，144DPI为24px；仅尺寸变化刷新，主题/显隐/TaskbarCreated仍显式刷新，HICON独立所有权。

## 已交付的新进程架构
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
- UiValidation保留原隔离同进程检查入口，只构建不运行，不能证明正式跨进程生命周期。前轮run42065045829及本轮run42215572575均已核对success，本轮正式工件已覆盖交付。

## 开发/发布/清理
- 活跃DSH session-c605187d-a18a-4ef4-8d4a-353f838e7a13，保留会话及源码直至明确项目结束；当前无DSH任务运行。
- 入口 AppWinMain.cpp、App.cpp、MainPage.cpp；IPC RadeonTunerIpc.h；backend RadeonTunerBackend.h；构建 .github/workflows/build.yml/windows-2025-vs2026，正式包先上传，后编译UiValidation，不在runner运行应用。
- 本轮开始本地3e516a8f77d7d81a1e3db648774e36b012556146、远端6a2b253b7d71fb6408969b435b211575ac994f61，同树5499310e7384c012268e5db184be3e5c4547a468。普通Git网络曾失败，API对象发布逐blob/tree核对，expected_sha非强制更新；不要盲推旧origin/main。
- .build-work/Prepare-GitHubUpload.ps1可重新生成临时上传清单；最新本地/远端/树映射保存于.build-work/evidence/source-publication.json，已用完的base64上传清单清理。下载放本任务.build-work，校验ZIP和EXE报告，再覆盖指定交付；清理可再下载ZIP/解压副本，保留发布证据/恢复文件。
- 本轮无本机编译工具安装。上轮VSBuildTools18.11.3实例98facc78/SDK26100/Installer已卸载；原.NET6/8/10、Desktop8/10、VC++v14 x64/x86及DOTNET_TieredCompilation=0保留。禁止重复跨会话outputs清理，禁止动其他项目/设置/共享运行库。
- 候选源码已发布：本地9e4c6873c44d696c9f14d517a2696b611e4884be、远端fad10e5b2462640cfe67e0b234221ce9303890c5，同树2d38d09495f269a6ee95bc5fba6835c25ed16108，16文件逐blob/tree校验。下一次API增量使用该本地/远端基线。最后3项关闭残留已由Codex修复并获限定复核通过；已进入云编译。用户不允许本机实测。
- 本轮云run42214476488 Release编译失败，唯一根因App::RefreshTrayIconForDpiChange漏头文件声明，引发11项连带成员解析错误。已补声明并核对全部App定义有对应头声明；不改实现/设计，需重编。首次候选本地9e4c687/远端fad10e5b树2d38d094。
- 编译声明修正已发布：本地e6299374b1f4426cc2c7a6433bfd79804ce52d4a、远端7b7f817b26abb8596b569eaf10a98376599e370e，同树fa30db154b7976930d9cd9c0f427b20ed7e3308b。下一次API增量以此为基线，当前等待第二轮云编译。
- 第二轮构建run42215572575（远端7b7f817b）正在Release编译；第一轮失败仅补了DPI方法声明，运行代码逻辑与已审候选一致。正式工件就绪后下载校验，严禁启动EXE。
- 正式Release已编译/打包/上传，run42215572575，artifact13501373058。已下载校验ZIP digest83EEA3EF5BBA5DAA6BC0FDCAAD8DC92280243D49E1EF0E1529C94FD9EDCE26AE，并覆盖指定EXE：3183616字节，SHA256 B093E80771CCD445F561A790EF9B46EFB2CC0E00489C3DC787589D81762DCCE1。打包报告与外部交付哈希一致，未启动任何应用。receipt与report在.build-work/evidence/process-split-*；下载ZIP/解压副本已清理。UiValidation仍在云端编译，未运行。

- 最终云run42215572575结论success：Release与UiValidation均编译/打包/上传成功，只做静态检查与编译打包，没有执行应用。代码交付对应本地e6299374/远端7b7f817b/树fa30db154；后续仅项目记录更新，不需要重编。正式EXE交付receipt及report保留，6.26MB下载ZIP/解压临时文件已删除；运行设置/源码/活跃DSH恢复资料保留待用户验收。

## 2026-11-07 数字输入交互修复
- 用户反馈数字框无法编辑，已确认滑块绑定TextBox使用ApplicationTextBoxNoHit（禁用hit test/Tab），仅作单向读数。本轮改可调数字为实际输入，Enter/失焦提交、Esc恢复；保留原Slider范围/步进及ValueChanged/Apply/save语义，真正只读和不支持字段不解禁。
- 继续云端编译并覆盖指定EXE，源码推私有仓库；依旧禁止启动软件实测。当前交付仍B093E807...3183616字节，未替换。
- 同DSH session-c605187d-a18a-4ef4-8d4a-353f838e7a13正在实施；本轮基线本地4616d1306bb8e7da954b566f8ce2afda13d56a17、远端13124116e49acc7da3535d40aa88eb0c1ef7c214，同树e434163b695d0f59a700b2352900fe945ea2117a，source-publication.json为准。保留原进程架构、DPI、界面几何/风扇算法/公开配置格式。
- DSH本轮启动失败，headless tools插件拒绝mode=ptc，sessionPersistence未激活；未恢复或更换会话、未改DSH配置。Codex已直接完成本轮实现：28个NoHit滑块读数改原生可编辑TextBox，GotFocus记录初值，Enter/失焦只在实际编辑后提交到原Slider.Value，Esc恢复实时OneWay绑定。完整有限数字校验、原范围/步进归一化、disabled与IsReadOnly不解禁，未编辑聚焦不改变原值。既有12个风扇输入保持不变，无新增驱动/IPC/配置语义改动。
- 真实检查：28项Tag/Value/IsEnabled/事件关联静态核对、汉化12项、布局7页104行、diff--check通过。源改动为App.xaml/MainPage.xaml/MainPage.h/.cpp及原布局检查预期，尚未云编译、没有运行软件。
- 数字输入修复已发布：本地f16c6883c...（完整值见Git HEAD）、远端388fd78d9b88e82a56554c032919a4a0efa0f2ae，同树6b0e91a64555153eff68a2b21dfb89d21f1333b3；下一API增量以本地f16c6883cb492435e3b0d7bf38b4842534a29a01、该远端为基线。当前等待云编译，交付EXE未替换。
- 追加同路径修复：UI snapshot Restore直接写Text/IsEnabled会替换Binding，已对Tag关联Slider的新数字框在快照恢复后重建值与能力OneWay绑定，避免重开/切换设备后失去同步。只改UI恢复层，IPC/公开配置不变。依据微软Dependency properties overview确认本地值覆盖绑定；当前首轮构建候选作废，需发布该追加后重编。
