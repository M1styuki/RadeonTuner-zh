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
