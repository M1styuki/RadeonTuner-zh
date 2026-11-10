# RadeonTuner 简体中文版：项目记录

## 用户目标与边界
- 当前交付版0.72已合并 dumbie/RadeonTuner 至99e759baa8701761fa75caf3b97e23d37ae0baf2（7次上游提交），保留仅简体中文、深色界面和现有后台架构；已完成本轮云端编译和EXE覆盖。
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
- 实测关闭worker后只剩host，但旧召回用ShowWindow显示host白色空窗，已可见时不再触发WM_SHOWWINDOW，阻塞再次打开。源码已将启动器和core两条召回改为注册消息RadeonTuner.ShowSettings，host走EnsureUI/LaunchUiWorker。该修复已合并最终界面EXE，最终包重开实测未完成，待用户验收。
- 当前交付D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe：0.72.0.0，3217408字节，SHA256 BD19A3C054FE532E467755D4EAF1D20D09A529901227829D2DDF4FBF6DDAD554；云run42831738936 Release与UiValidation完整success，仅编译打包，未执行应用。最终receipt/report为.build-work/evidence/upstream-42831738936-*。
- DSH首轮视觉包仅改App.xaml和verify-ui-layout.ps1；diff/汉化12项/7页104行等静态检查通过，无GUI/本地编译。Codex针对报告未决项发现Indicator三态空及PanningRoot状态不完整，已给同会话定向修复，参考微软release/2.8的ScrollBar_themeresources.xaml；修复已完成并通过静态检查，不重复已有效检查。

## 开发、发布与恢复
- 活跃DSH session-c605187d-a18a-4ef4-8d4a-353f838e7a13，同一cwd复用。本轮视觉包已成功返回，旧启动配置错误未复现；不能据此宣称PTC节省已验证。保留活跃历史，不动全局DSH链路。
- 代码入口AppWinMain.cpp/App.cpp/MainPage.cpp；IPC RadeonTunerIpc.h；后台RadeonTunerBackend.h。云构建.github/workflows/build.yml，windows-2025-vs2026，正式Release工件先上传，再编译UiValidation；云端不运行程序或驱动。
- 最终应用代码：本地2e7232432836eb6d804a01d95f48527faad900c8、远端5b74baf1babf42b8950bdd2253daa0748a533b79、同树1fc9402de8beba825bfca4c5a37d014f0e026efb，run42359458442 success。后续只有项目记录skip-ci提交，最新完整本地/远端基线见source-publication.json；origin/main陈旧，勿盲推。
- .build-work/Prepare-GitHubUpload.ps1基于已提交Git原始blob生成临时base64清单；发布逐blob/tree校验，expected_sha非强制更新。最终本地/远端/树映射写入.build-work/evidence/source-publication.json。项目记录更新用skip-ci提交。
- 验证入口tools/verify-localization.ps1、tools/verify-ui-layout.ps1、git diff --check；只做受影响检查。不要把构建、静态或一次编辑等同于全部驱动/内存/生命周期验收。

## 清理边界与环境
- 前两轮cloud-keyboard/cloud-panel-input共8个临时下载/解压文件约12.6MB已删除，build-report等证据保留；cloud-recall和cloud-interface共8个临时文件已在本次收尾删除，EXE已在外部指定目录；构建报告与交付receipt保留。只清理本任务验证过的临时文件，不跨会话批量清理outputs。
- 本轮无本机编译环境安装。上轮VSBuildTools18.11.3实例98facc78/SDK26100/Installer已卸载；原.NET6/8/10、Desktop8/10、VC++v14 x64/x86及DOTNET_TieredCompilation=0保留。不可误删共享环境、其他项目、凭据、用户设置或活跃DSH历史。
- 收尾前必须先验证指定EXE及用户要求的GitHub源码已成功保存；项目只有用户明确满意结束后才删除本地源码/Git/项目记录及本项目独有环境。当前不满足关闭项目条件。
- DSH定向修复已完成：按微软WinUI2.8核对Grid/Panning Border部件、命名轨道与Thumb Transform，补齐Indicator三态及ConsciousStates，实际使用hover/pressed色，保留ScrollBarStyle/DefaultScrollBarStyle/隐式入口。XML解析、汉化12项、原7页104行等布局断言和新增原生部件/状态断言、diff检查均通过。其后Codex云构建已成功并覆盖EXE；最终GUI检查未完成，外观由用户验收。
- 当前视觉代码已发布：本地2e7232432836eb6d804a01d95f48527faad900c8、远端5b74baf1babf42b8950bdd2253daa0748a533b79、同树1fc9402de8beba825bfca4c5a37d014f0e026efb；云run42359458442已success。后续API增量以这组为基线；source-publication.json已更新，交付及限制已记录，完整最终基线见source-publication.json。

- 最终GUI验证限制：准备退出旧版期间Sky检测到用户正在操作，Codex停止Computer Use；未强杀当前host/worker、未修改运行目录/用户设置。外部EXE已覆盖，但当前仍运行旧core。用户需从应用内选择退出后再打开EXE完成升级；不能宣称最终重开、滚动条或触摸交互实测通过。先前相同数值输入代码的删除/键入/Esc实测有效。

## 2026-11-10 用户更新后的下载覆盖
- 远端main已由用户更新到4f094ace8cc01ad9673621d88762bca1e19c967f，run42765738420 success；已下载正式工件13740755217并覆盖D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe，3204096字节，SHA256 4F3021903931FEE5BC18A0198C92E8BBF03369D62CA7E9D239591AE00F360BD5，与工件ZIP digest及打包报告一致。
- 本次只下载覆盖，没有启动程序/GUI测试/修改设置/重新构建/推送源码；下载及解压临时文件已清理，报告与交付receipt保存于.build-work/evidence/update-42765738420-*。
- 本地源码仍处于前次提交3f2ec5b；source-publication.json中的本地/远端映射仅为历史基线。未来开发需先与用户新远端同步，不得用旧本地树覆盖用户更新。

- 2026-11-10用户明确授权合并最新上游、提交GitHub、云构建并覆盖EXE。Codex已同步origin/main 4f094ac并核对应用代码完全保留用户导航初始化修复；本地fec8da7仅多项目记录，远端4f是祖先，可正常Git推送，不再用旧API增量基线。上游已fetch固定99e759baa8701761fa75caf3b97e23d37ae0baf2，7次提交交同一DSH会话以真实merge --no-commit合入，Codex最后提交。
- 合并设计：保留当前平铺汉化深色/原生滚动条/可编辑数字/隐藏托盘/独立轻量host+按需worker。范围风扇新增曲线选项兼容旧配置并纳入完整内部元数据交接；Gamma按实际API目标跟踪本程序写入归属，只实际写入路径更新，并随host/worker交接，关闭且未控制过不写Gamma，启用后关闭恢复一次，草稿切换不冒充实际状态；服务启停仅手动及原管理员门控，不运行服务/驱动/GUI测试。此包涉及ADL reset与运行事实，需要非DSH独立定向审查，然后云编译。
- DSH合并包已完成并暂存，MERGE_HEAD=99e759b，当前HEAD仍fec8da74be5e622ab6751c14bdecd1dd5bb87a4b；8处冲突已解，未提交推送/GUI/本地编译。新增gammaAppliedTargets运行归属随内部gamma_restore_targets交接，FanRangedCurve完整元数据/默认兼容及手动服务SCM辅助已加入。DSH实际检查：cached/working diff、汉化12项、布局7页106行/24下拉/4按钮行、XML及声明括号核对通过。Codex统一core和launcher元数据0.72.0.0，core resource.rc保留原UTF16LE BOM与原长度1146字节。独立merge_safety_review正在只读核对Gamma失败/交接/线程、OD8 reset、曲线字段和服务手动权限路径；通过后提交推送正常Git，并云构建覆盖。
- 独立复核未通过候选，已给同一DSH定向修复5项：AdlSettingInt/Float/String重复Get签名(P1编译阻塞)；Gamma型号级key串目标；两次Gamma写入+归属更新未整体串行且成功判断读共享adl_Res0（改本次局部返回值）；范围风扇匹配只比较端点而漏Linear/Quiet中间点；服务startService参数方向反。审查其余OD8 reset/count、FanRangedCurve默认与完整IPC、正常暂停交接未见额外问题。修完仅复核这些路径和新改动，再提交构建。
- DSH已修完独立复核5项：三个重复Get收敛为单个fallback实现；Gamma键改完整PNP/UDID/PCI+实际adapterIndex，未知完整身份拒绝写而非共享；新增gammaWriteMutex串行两次写入/归属更新，锁内重查恢复资格并用局部ADL返回值；Apply/Match共享五点曲线公式；服务传!disableServices，配置和启停分别统计并报告部分失败。汉化/布局106行/24下拉/4按钮行、目标diff、XML和静态核对通过，仍未运行驱动服务或编译。独立复核正在只看修复闭合，尚未提交/推送。

## 2026-11-10 上游0.72合并待交付
- 已合入上游7次提交，保留用户4f094ac导航初始化修复、现平铺汉化/原生滚动条/可编辑数字/隐藏托盘及轻量双进程架构；增加范围风扇线性/静音、修复默认风扇恢复、关闭Gamma保护ICC、手动Radeon服务控制及相应中文说明。core与launcher版本均0.72.0.0。
- 独立复核5项已修复并闭合，最后一处锁外Gamma预筛选由Codex做最小修改移除：关闭分支总进入AdlGammaApply(...true)，写锁内核对归属，无归属不写驱动。reviewer仅复核新改动确认无剩余直接阻塞；未做真实驱动/服务/GUI验证。DSH相关静态检查已通过，保留结果不重复无关验证。
- 下一步本地真实merge commit、普通Git快进推送（origin/main 4f094ac为祖先），云端Release/UiValidation编译，下载正式单EXE覆盖指定目录；不启动应用、不改本机设置/服务，不根据旧指令关机。
- 源码合并提交已发布：本地72ebd795330f2300f110f8226c1b911e671607ea（真实merge含99e与用户4f祖先），普通Git推送两次github:443连接失败；改API逐34个blob及整树校验，远端126642543376001cf4010968c9a5dc1c2588b0e5与本地同树edbccc96c9b9fdab3eed3b1d2a2456985a7ee4a0。API提交为单父4f的等价内容提交，不含上游99e原始祖先；本地真实merge历史保留供后续上游合并。后续API增量须用这组本地/远端同树映射，不能误认为SHA相同或盲推覆盖远端；source-publication.json已更新。当前等待云编译。
- 本轮正式Release编译/打包/上传成功，云run42830308665，artifact13770289132，ZIP digest e285f15094b1046c3a11d7e8ba0613cd2a34a797499927d4da855698ac3385d8。已下载核验并覆盖指定EXE：0.72.0.0，3217408字节，SHA256 FB249E3DCFD9BE2D6519AC8B4BF3FCC323EEC6F7F91222832F0701F9C2BA6882，与打包报告一致；未启动程序/驱动/服务、未修改AppData设置或运行文件。本轮4个临时下载/解压文件已清理，upstream-42830308665-build-report.json和delivery.json保留。该次UiValidation后续因测试工程旧控件名编译失败，见下方修复与最终成功记录。
- 首轮云job42830308665整体failure仅发生在后续UiValidation编译：UiBehaviorTests.h和UiValidationScenario.h共5处仍引用上游已更名的combobox_FanControl_Mode/事件名，正式Release已成功交付。Codex按当前MainPage声明做纯字面更名为combobox_FanControlMode，保留原UTF8编码/BOM/换行，无生产代码改动；diff核对及旧名称全量检索通过。需提交后再云编译，不能将首轮完整job写为success。
- 隔离编译控件名修正已发布：本地20cef0a3f96d215817683102c797525069592b7a、远端a994bc61f87b8a847ebfd2fcef6f34d45a333b0a、同树0fb17fc65462276fdbf4f2276be1168abe18f594。已验证RadeonTuner生产源码目录树在72ebd79与20cef0a间完全不变（e538eddddcd7bbf9f27e4505aafdce51c4e4f2ce），故此前正式EXE/独立复核仍有效，不因测试工程更名重复改动驱动代码。等待新job完整编译结果；后续API增量用20cef0a/a994bc映射。

## 2026-11-10 上游0.72最终交付完成
- 最终云run42831738936完整success：正式Release和UiValidation均编译/打包/上传成功；未执行GUI/驱动/服务。首轮隔离工程旧控件名已修复，生产目录树与已独立复核版本相同。
- 已用本次最终正式工件13771972322覆盖唯一交付EXE，0.72.0.0，3217408字节，SHA256 BD19A3C054FE532E467755D4EAF1D20D09A529901227829D2DDF4FBF6DDAD554；ZIP digest c56f2bb5319626e2e25a90a71fb4ed088bdac1f4be18b4357451ba6c9ca16bc0，与打包报告和外部文件匹配。未改AppData设置/运行文件，用户正常退出旧程序后重新打开即可完成运行文件升级。
- 本轮两次下载共8个临时ZIP/解压/日志/副本已删除；receipt与报告、活跃源码/测试/Git/DSH会话/审查记录保留待用户验收。无本机开发环境安装。
- 最终应用及隔离更名修正对应本地20cef0a3f96d215817683102c797525069592b7a、远端a994bc61f87b8a847ebfd2fcef6f34d45a333b0a，同树0fb17fc65462276fdbf4f2276be1168abe18f594。后续仅项目记录skip-ci提交，最终映射见source-publication.json；生产目录树e538eddddcd7bbf9f27e4505aafdce51c4e4f2ce，已合并上游至99e759b，并保留用户4f导航修复。
- 2026-11-10用户反馈页面右侧滚动条离边缘过远或压住控件。定位主ScrollViewer外侧右Margin16而内容内侧仅8，16DIP滚动条命中区域会侵入内容。Codex直接修正两处XAML边距：外Margin14,8,0,0，内容Grid Margin0,0,24,18；总右侧内容距离仍为24，因此所有控件/风扇图原右边界不变，滚动条向右16且与控件留8DIP间隙。仅主页面滚动区域，不改下拉/模板/驱动/后台；汉化12项、原布局7页106行等及diff检查通过，无GUI实测。按现有云构建/覆盖流程交付，源码基线本地8f86da5/远端49f8431同树。
