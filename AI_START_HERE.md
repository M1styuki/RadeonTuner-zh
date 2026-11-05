# RadeonTuner 简体中文版：项目记录

## 交付与收尾决定
- 用户已明确要求本轮编译后提交源码至私有仓库 https://github.com/M1styuki/RadeonTuner-zh ，核验成功后清理本项目本地源码、Git、测试副本、构建产物、项目DSH会话与专用开发环境。
- 保留交付 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe，以及运行文件、用户设置和原有/共享运行库。禁止清理其他项目、重新执行跨会话 outputs 清理或再次关机。
- 正式单EXE 0.70.0.0，3150848字节，SHA256 D14F1AEAE7058B6D2A4FD5CBDC712A93A381DA851D5B98A10D850A2D5FBE6735。
- 核心 SHA256 022D1F52FDD1514764F6F20BF021477CBCAAEFFB323DE5BED11DAE77F7FE6D5F。只解包 verified=true、launched=false，解包核心与 Release 一致，外部交付副本哈希一致。

## 目标及实现边界
- 上游 dumbie/RadeonTuner 官方0.70，合并提交 dfade6b，包含上游 c120f54a3b93ef07e150c15e50553b877e857638。保留原许可证；本仓库仅私用，不公开再许可。
- 仅简体中文，深色界面，原生中文深色右键菜单，不使用自绘菜单。隐藏托盘为唯一新增功能；其余保留原功能与驱动设置语义。
- 无卡片/分组小标题/装饰分隔线；左侧说明与 ? / !，右侧控件共用右边缘。常规数值和下拉框160x32 DIP，数字居中；普通滑块等长。
- 监控13项使用透明5列3行网格，不新增采样、传感器或历史；显存时钟显示MHz。
- 风扇图完整外框与细网格，五点温度/转速成对数值输入；最低/最高转速同行保留滑块。真实条件项开关控制折叠，页面和下拉框保留短过渡。
- 开关44x32：保留原生键盘、自动化及拖动输入。补齐原生DoubleTapped漏掉的第二次切换；可见圆点使用独立100ms动画，从当前模板可见位置即时接续。关闭/反向不排队、不防抖、不禁用连点。原生Thumb拖动同步到可见圆点。
- 后台改动仅保留此前获准的分配优化。不添加驱动兼容修补、模拟驱动失败、配置迁移或其他功能。

## 入口与验证
- 构建、运行和设置目录见 README.zh-CN.md、tools/Portable/README.md。
- 界面：RadeonTuner/MainPage.xaml、App.xaml；动画：MainPage.cpp 中 AnimateToggleKnob、toggle_Loaded、ApplyDependentRow。
- 验证：tools/verify-localization.ps1、tools/verify-ui-layout.ps1；隔离版添加 /p:UiValidation=true，入口 -ui-animation-check。
- 最终 Release 与 UiValidation 构建均成功；汉化12项和7页布局检查通过，diff --check通过。
- 最终隔离检查12次实际鼠标连点各切换一次；反向样本18.33→12.28→7.73→3.26→1.53→0.29，最终归零。普通开关、条件行折叠、图表边界检查通过。Computer Use另核对风扇零RPM开关拖动可切换；未另测键盘操作。
- 隔离版在加载ADL前阻断，禁用用户设置保存；本轮没有普通启动正式核心、执行驱动测试或读取用户设置。真实驱动全功能仍非本轮验收结论。
- 无有效同场景完整程序RSS前后实测，不宣称具体内存降幅。

## 开发与清理归属
- DSH会话 session-9c436592-18c9-4133-8cd5-59aeaac2abe6 本轮20分钟超时已停止，Codex完成最终修复；不使用DSH审查。本地会话历史按用户收尾授权清理，聊天可见调用无删除API。
- 本任务新增 BuildTools18.11.3实例98facc78、MSVC14.51.36231、SDK26100/UWP构建支持及其.NET开发组件；优先使用官方卸载器清理任务专用实例。
- 原有.NETCore6.0.36/8.0.29/10.0.6、WindowsDesktop8.0.29/10.0.6及VC++x64运行库14.51.36247保留；DOTNET_TieredCompilation=0为原有设置。
- 本任务work含源码、baseline/界面测试副本、下载、NuGet私有缓存与安装前后清单。baseline/packages是指向本项目packages的目录联接，清理时先解除联接，不能遍历链接删除。
- VS安装暂存 %TEMP%\5jiqdgfs；安装日志前缀20261102235155/235346/235422/235424和20261103000401/000402/000405/000544，以及dd_vs_BuildTools_decompression_log.txt属于本任务。其他日志、共享缓存及凭据保留。

## 2026-11-05 新一轮：按需释放界面与透明图标
- 上轮已按用户授权完成源码上传及本机开发环境清理；现从 M1styuki/RadeonTuner-zh main 的489fa50恢复同一路径。旧DSH本地历史已在明确项目收尾时删除，本轮有意新建会话，不能续接已删除ID或重新安装本机编译器。
- 用户新增要求：关闭设置面板后不再保留整套界面；去掉应用图标底色，仅保留白色图形；仅云端编译，下载后覆盖 D:\Download\RadeonTuner-zh\RadeonTuner-简体中文版.exe。原有界面布局/功能/设置文件保持，原交付及用户数据在新版验收前保留。
- 当前80MB解释已核对源码：App::Exit(false)关闭托盘分支只SW_HIDE，未卸载XAML；监控循环隐藏时跳过采集，但MainPage及XAML树仍被Content持有；配置保持/应用检测循环仍每10秒或被请求后2秒运行。未测量各部分内存占比，不把80MB全部归为泄漏。
- 批准实现方向：同进程，App持有后台任务与配置状态，UI为可释放会话；后台不得持有MainPage原始this。关闭时停UI定时器/动画、断开Content并Close DesktopWindowXamlSource，释放MainPage及控件/图像引用；重新打开创建UI并从已有状态恢复，不能重复驱动初始化/默认值写入/后台线程。保留WindowsXamlManager和线程Dispatcher到线程结束；微软明确不支持在同一线程Close后重新创建manager。后台回调经UI线程解析当前弱页面，只允许更新当前会话。保留保持生效、应用配置切换、Gamma/Eyefinity、托盘/隐藏托盘/再次EXE唤回及未应用草稿语义。
- 需证明：旧页面弱引用释放，反复开关无累积页面/线程/定时器，后台功能不依赖页面存活；隔离UI样本测同进程打开/关闭内存，不能伪装成真实驱动全面验收或承诺固定降幅。严禁普通启动正式核心做测试，严禁新增驱动失败/配置迁移模拟。
- 云端构建优先官方runner windows-2025-vs2026（VS18/v145、SDK26100），保留Release/UiValidation分离以及现有硬阻断。原动画连点检查保留，生命周期只测此次变化。图标由Codex另行处理，DSH不改图标资源。
- 图标已获用户明确同意改为程序按原图像素去底，不再生成/重绘。原AppIconColor.ico的256PNG帧提取白色分量，全部原有实心白色像素保持不变；两连通主体及抗锯齿边缘保留，RGB为白色、背景alpha=0、心形内部alpha=255。替换AppIcon.ico/AppIconColor.ico/AppIconMono.ico，内含256/64/48/32/24/16尺寸；原生小图标解码与深色底预览已检查。两次生成式图像产物质量不合格，不纳入源码或交付。本轮仅临时提取脚本和预览在work/icon-review，完成后清理。
- 当前DSH新会话 session-c605187d-a18a-4ef4-8d4a-353f838e7a13：首包25分钟超时停止，第二包同会话定向完成4项独立审查修复（picker旧会话取消、Overlay不进快照、计时器单次Tick注册、指标查询恢复后台并校验expectedSession）。已做汉化12项/布局7页104行/diff静态检查，尚未云编译和实际运行。独立审查代理正在复核；Codex已补MetricsLoop遗漏的MainPage using。
- 用户确认性能数据/图形状态/所选设备检查可在面板创建后再采集，关闭即暂停；配置保持/应用配置/Gamma/Eyefinity继续后台工作。后台仅保留必要框架基础，释放的是页面/控件/岛，不宣称框架全部卸载或具体RAM降幅。
- 云构建配置使用windows-2025-vs2026，Release与UiValidation工件分离；GitHub已有Git凭据具备repo/workflow权限。将提交并触发首次云端编译，不在本机重装VS/SDK。
