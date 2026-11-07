# RadeonTuner 简体中文版：项目记录

## 用户目标与边界
- 基于 dumbie/RadeonTuner 0.70，已合并 dfade6b/c120f54a；仅简体中文，深色界面，保留原驱动调节、配置与响应行为。
- 私有源码仓库 https://github.com/M1styuki/RadeonTuner-zh 。用户明确授权上传源码、仅云端编译、下载覆盖 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe。最初只交付一个EXE，允许首次运行释放必要运行文件和设置。
- 运行根 C:\Users\78539\AppData\Local\RadeonTuner-zh-CN\App；Settings\Settings.json、Profiles及用户数据必须保留。不得用迁移/驱动失败模拟扩大验证。
- 用户现授权Computer Use复现/验证数字编辑及面板关闭后重开，替代此前不实测限制；只操作本应用，在原值/撤销范围内，不点Apply，不改驱动设置，不测性能。用户同意新版就绪后约20秒操作；其他工作不用Computer Use。外观最终由用户验收。
- 旧关机指令已于前轮执行，用户现在清醒使用电脑，不能再次自动关机。源码/测试/会话仅在用户明确满意并结束项目后清理；当前仍在开发。

## 已确认的界面要求
- 无卡片、无重复大小标题、无行分隔线；左侧正式简洁说明及问号/叹号，右侧控件共用右边界。输入/下拉160x32、数值正中、等长滑块、等行距；切页不改窗口大小。
- 监控13项、5列3行；风扇图完整边框、规整细网格/坐标；温度/转速成对数字输入，最低/最高转速滑块放底部。ECC等不支持控件仍禁用。
- 条件项折叠与短动效，开关快速点击可即时反向；原生中文深色右键菜单，不重绘菜单。隐藏托盘功能保留。
- 新增视觉包使用用户安装的 b-end-design-system：采用设置页左分类+右平铺，保留上述用户覆盖规范的约束；统一冷灰深色、柔和红强调#C85C63、6px控件/8px浮层圆角、字号/留白/低对比边框。
- 页面边缘悬浮展开滚动条及下拉内滚动条两者都调整：细窄圆角滑块、透明轨道、16px命中区。复用原生滚动行为，不引入新组件/图标依赖或后台定时器，不改变设置立即生效/Apply语义。
- 白色透明图标按原像素去底，ICO含256/64/48/32/24/16；托盘按任务栏DPI选择尺寸，144DPI为24px，仅尺寸变化或原主题/显隐事件刷新，独立HICON所有权。

## 保留的进程与数据架构
- 一个core具有原生host和-ui-worker双角色，后台不初始化XAML；面板按需创建独立进程，关闭即退出，后台保持/应用配置匹配/Gamma/Eyefinity继续运行。性能采集仅面板存活时运行。
- host与worker各建ADL上下文；host每次生命周期仅一次默认初始化/应用同步。开worker前等待host写入批次暂停，worker期间host暂停；worker退出前等待其批次和异步UI操作结束，避免同时写驱动。
- 内部匿名双向pipe、固定长度头、8MB上限、STARTUPINFOEX继承句柄清单；模型与草稿只存内存，不新增公开配置格式/永久会话文件。完整模型含Current/Default/范围/Step/Support/UsingProfile，optional字符串null保持。
- 每次开worker刷新host实际生效flags，按DeviceId+Application合并草稿；后台实际生效cache与UI选中草稿分离。设备身份失配不恢复错设备snapshot。
- worker结束后host重读设置/Profiles并恢复实际生效档；AVSettingsJson缓存整文件，交接重新构造，托盘显隐/主题主动通知，无200ms整文件读盘。
- host/worker消息按所属HWND/PID核对，快召回排队；完整退出异步握手，继续读pipe，等待真实分辨率回滚与Custom删除完成。Job kill-on-close防孤儿，SILENT_BREAKAWAY_OK允许Updater/外部工具脱离，更新成功走整套退出并co_return。

## 当前修复与验证
- 28个滑块数值框已从假只读模板改原生可编辑ContentElement；Tag绑定原Slider，Text/IsEnabled实时绑定。Enter或失焦提交完整有限数值，按原范围/步进归一化；Esc恢复原值，未编辑聚焦不改值。仍走原ValueChanged/Apply逻辑，既有12个风扇整数输入保留。
- UI循环已补IDesktopWindowXamlSourceNative2::PreTranslateMessage，native frame获得WM_SETFOCUS时SetFocus到XAML并NavigateFocus(Restore)。快照Capture/Restore限定page.FindName对应对象，排除跨控件同名模板部件，重建后恢复数值绑定。
- 已实际观察：运行core F6B7EDFBA833A55D6164C80C7ACFB56E4B8BE4424B6FA1E0E22B6B0DE4422B85，风扇Temp0可删除为空再输入30；调校Core_Max输入1时Slider仍0，Esc后均0，未点Apply。Sky立即抓取可能早于消息处理，必须稳定后再判断；此前仅立即返回旧文本的失败观察不能证明修复无效。
- 实测关闭worker后只剩host，但旧召回用ShowWindow显示host白色空窗，已可见时不再触发WM_SHOWWINDOW，阻塞再次打开。源码已将启动器和core两条召回改为注册消息RadeonTuner.ShowSettings，host走EnsureUI/LaunchUiWorker。该修复已云构建，待与新界面合并后实测召回。
- 当前交付目录仍为58AF0B6D160BE3A1C8688A9CABE0A8E267C00BD2AA32F5D7E17F4EAF343D03D1/3203072字节版。后续正式包已下载在.build-work/cloud-recall，SHA256 2E9C1901A7ABE8FC81DA388F7565700F95228C42755278E600C343092BE0F255/3201024字节，尚未覆盖，不能把待交付包描述为运行中。
- DSH首轮视觉包仅改App.xaml和verify-ui-layout.ps1；diff/汉化12项/7页104行等静态检查通过，无GUI/本地编译。Codex针对报告未决项发现Indicator三态空及PanningRoot状态不完整，已给同会话定向修复，参考微软release/2.8的ScrollBar_themeresources.xaml；当前等待修复结果，不重复已有效检查。

## 开发、发布与恢复
- 活跃DSH session-c605187d-a18a-4ef4-8d4a-353f838e7a13，同一cwd复用。本轮视觉包已成功返回，旧启动配置错误未复现；不能据此宣称PTC节省已验证。保留活跃历史，不动全局DSH链路。
- 代码入口AppWinMain.cpp/App.cpp/MainPage.cpp；IPC RadeonTunerIpc.h；后台RadeonTunerBackend.h。云构建.github/workflows/build.yml，windows-2025-vs2026，正式Release工件先上传，再编译UiValidation；云端不运行程序或驱动。
- 最新已发布代码：本地afe69011f746ddf361d815eac6a3d2107e9736da、远端7d139ca1d7891b4d5c7692411df423bc2cb2a74d、同树b68360d68f1ffa36271f0ef8d5ccccb075d55498，run42357578668正式产物已核验。下一次API增量以此本地/远端为基线；origin/main陈旧，勿盲推。
- .build-work/Prepare-GitHubUpload.ps1基于已提交Git原始blob生成临时base64清单；发布逐blob/tree校验，expected_sha非强制更新。最终本地/远端/树映射写入.build-work/evidence/source-publication.json（当前其中旧映射尚待更新）。项目记录更新用skip-ci提交。
- 验证入口tools/verify-localization.ps1、tools/verify-ui-layout.ps1、git diff --check；只做受影响检查。不要把构建、静态或一次编辑等同于全部驱动/内存/生命周期验收。

## 清理边界与环境
- 前两轮cloud-keyboard/cloud-panel-input共8个临时下载/解压文件约12.6MB已删除，build-report等证据保留；cloud-recall留待最终交付后删。只清理本任务验证过的临时文件，不跨会话批量清理outputs。
- 本轮无本机编译环境安装。上轮VSBuildTools18.11.3实例98facc78/SDK26100/Installer已卸载；原.NET6/8/10、Desktop8/10、VC++v14 x64/x86及DOTNET_TieredCompilation=0保留。不可误删共享环境、其他项目、凭据、用户设置或活跃DSH历史。
- 收尾前必须先验证指定EXE及用户要求的GitHub源码已成功保存；项目只有用户明确满意结束后才删除本地源码/Git/项目记录及本项目独有环境。当前不满足关闭项目条件。
- DSH定向修复已完成：按微软WinUI2.8核对Grid/Panning Border部件、命名轨道与Thumb Transform，补齐Indicator三态及ConsciousStates，实际使用hover/pressed色，保留ScrollBarStyle/DefaultScrollBarStyle/隐式入口。XML解析、汉化12项、原7页104行等布局断言和新增原生部件/状态断言、diff检查均通过。未编译或跑GUI；下一步Codex云构建并覆盖，约20秒验证数值编辑与重开，外观由用户验收。
