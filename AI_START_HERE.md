# RadeonTuner 简体中文版：项目记录

## 当前目标与交付
- 私有源仓库 https://github.com/M1styuki/RadeonTuner-zh ，用户授权源码上传、仅云端编译、下载覆盖 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe；最终仅此一个EXE，首次运行可释放必要运行文件与设置。
- 2026-11-06 新要求：4K+150% DPI托盘图标模糊；后台约50MB仍过高。用户明确批准“轻量原生后台＋按需独立面板进程”，保留超频、降压、功耗、风扇及原功能。
- 最新用户明确“不用实测，我自己能看”：禁止自动启动/操作应用、GUI/驱动/内存实测和模拟；只相关代码检查、云编译、下载覆盖，由用户验收。
- 前轮关机指令已成功下达，现用户已恢复使用电脑，本轮不自动关机。源码/活跃DSH历史保留至用户明确满意并结束项目，不能依据旧收尾授权删除当前开发项目。
- 最新EXE已覆盖并校验：3201536字节，SHA256 0BF54F22AAABDE10B9C08FD09D97300784FC57FC481913A2142C6918573A616E；2026-11-07数字输入修复，run42353651555的Release编译/打包成功，未启动应用。
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

## 2026-11-07 数字输入交互修复：已交付
- 用户反馈数字框无法编辑，确认28个滑块绑定TextBox用了禁用hit test/Tab及隐藏ContentElement的ApplicationTextBoxNoHit。已改原生可编辑模板，保留160x32/数值居中等视觉与布局。
- 支持点击/选择/粘贴/Tab，Enter或失焦提交，Esc恢复当前值；编辑中允许空白/负号/小数，不逐键重置。只提交完整有限数字，按原Slider范围/步进归一化，再走原ValueChanged路径，不改变原应用按钮/保存/驱动语义。未编辑聚焦不改值，Enter后失焦不重复应用。真正只读/不支持字段保持，既有12个风扇输入未变。
- 提交/取消及UI快照恢复后重建Text/IsEnabled实时OneWay绑定，避免重开或换设备后失去同步。仅UI恢复层改动，后台/IPC协议/公开配置格式未变。
- DSH本轮因tools配置mode=ptc不兼容、sessionPersistence未激活启动失败；未修改DSH配置或重新建会话，Codex直接实现。活跃session-c605187d-a18a-4ef4-8d4a-353f838e7a13及其历史保留。
- 实际检查：28项Tag/Value/IsEnabled/事件关联静态核对，汉化12项、布局7页104行、最终diff检查通过。修正过Data::Binding/BindingMode命名空间后，云run42353651555正式Release编译/打包/上传成功。隔离版在正式交付时仍在云端构建；未运行任何应用、驱动、内存或UI测试。
- 交付 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe，3201536字节，SHA256 0BF54F22AAABDE10B9C08FD09D97300784FC57FC481913A2142C6918573A616E。ZIP digest59F1F050A05DBFAAC956B7A2C8C086F66DF9B0F258FD62E5556F46355FAE3B66，artifact13566568061；与打包报告及外部副本一致。
- 应用代码本地612e41de124813e858f79b24d311afd652ccbefb、远端051e60f0cc0610849bdc9dc66457406c24304361，同树5c616a68267500bab3f5c4e0491259735a22cf02。最终项目记录为单独skip-ci提交，最新映射见.build-work/evidence/source-publication.json，后续API增量以该映射为准。
- 4个临时下载/解压文件已清理；numeric-input-delivery.json及numeric-input-build-report.json保留。源码/设置/活跃会话等待用户验收，禁止自动启动软件或继续套用旧关机指令。后续若需更多修改，复用现库与会话；DSH仍不可用时明确fallback，不擅自修改全局DSH链路。
## 2026-11-07 用户再次反馈：只能选中，不能输入
- 已只读核验外部EXE SHA256=0BF54F22...，运行App目录core=DFFDADECF90A1A96E86859ACF156189DAEC162A8EC7E9A9858F99E3E1EC960DB、Resources.pri=58E0BB550E87D67F3252C87D5C621A3EC4070F8B3BE268F5065E53BBB9D1B1DF，均与numeric-input-build-report一致，确实已更新，不归因为用户没重启。
- 用户明确新授权使用COMPUTER USE，覆盖此前不实测限制，仅复现/验证数字编辑，不点击应用，不改驱动设置。已用Sky复现风扇Temp0：点击后焦点在textbox_Fan_Temp_0，Ctrl+A/BackSpace未改变30；没有点击Apply，也未改变数值。首次召回另观察到native host空白窗，随后实际worker正常出现；此为已观察相关宿主问题。
- 新定位：CreateWindowXaml消息循环只有TranslateMessage/DispatchMessage，没有IDesktopWindowXamlSourceNative2::PreTranslateMessage，也无WM_SETFOCUS/NavigateFocus处理。微软UWP XAML Islands文档明确需要前置键盘消息路由。下一步补宿主输入管线，云编译后用Computer Use在原值/撤销范围验证真实键盘编辑，不能仅凭编译宣称解决。
- 本轮直接补UI工作进程消息循环PreTranslateMessage：仅XAML已处理消息跳过后续分发，未处理/失败走原Translate/Dispatch；每条消息临时解析当前source，避免持有已释放island。原生后台循环不初始化XAML、不改驱动/配置。待云编译后Computer Use验证实键Ctrl+A/删除/输入/撤销；未改动当前运行数值30。
- 键盘宿主修复已发布：本地f3fc267a191f9c3d8da984003b11dd9272adc2ec、远端905a5ec3ef603f27374331b4a1509c541856e4bd，同树c2e57c7be49a6a1c22693422d8ed597b8888ecc8；后续API增量以此为准。当前云编译，交付仍是已确认存在键盘问题的0BF54F22版，待新版真实验证。
- PreTranslate单独修复run42355167410正式包已覆盖并核验运行core=B480AC7D...；Sky实测Temp0依旧无法Ctrl+A/BackSpace/输入1，值仍30。不能宣称已修复。新增WM_SETFOCUS将native frame键盘焦点交还XAML子窗口，并NavigateFocus(Restore)保留当前编辑目标，待第二次云编译及实测。
- 用户指出重建面板前输入正常。已对比489fa50与当前：风扇模板未改，窗口由预先可见改为延迟显示；另发现快照递归视觉树按Name单键保存，包含跨控件重名模板部件（ContentElement等）。Capture/Restore现限定page.FindName同一对象，避免内部模板状态串写；需验首次与重开后的键盘输入，不能把该缺陷直接当作已证实的唯一原因。
