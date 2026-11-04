# RadeonTuner：续做入口

## 目标与边界
- 用户要求仅简体中文，保留全部功能/响应与操作结构；精修前端、优化后台内存；默认深色、中文深色原生右键菜单，明确禁止自绘/owner-draw。
- 新增 HideTrayIcon（默认false），隐藏不停止后台功能，再启动EXE唤回；设置持久化。
- 2026-11-04用户否定卡片和分组小标题：所有主页面去掉卡片背景/边框/圆角及GPU调校等小标题，直接使用完整字段名（如GPU最高频率偏移、显存最高频率）。保持功能和操作顺序，扁平行列表、统一留白/细分隔线；原生输入控件及弹窗正常外观保留。正文13、辅助12、弹窗问题14、控件32高。
- 每项顺序统一为文字说明→问号→叹号→弹性空白→右对齐开关。问号/叹号跟在文字后，不能继续在左边。滑块/下拉项也补全名称、保持一眼能分辨GPU/显存/功能所属；原x:Name、事件、警告显隐和设置语义保留。
- 用户启动时收到更新提示，明确要求合并官方最新更新；已查官方最新发布v0.70.0.0，2026-11-04香港时间更新，最新main c120f54a3b93ef07e150c15e50553b877e857638。正在fetch和核对上游差异，合并后继续保留中文/深色原生菜单/隐藏托盘/内存优化/单EXE。
- 本地构建已授权；首次交付只有一个EXE，运行后可生成配置及必要运行文件。不发布/上传源码。最终输出 D:\Documents\codex\2026-11-02\jia\outputs\RadeonTuner-简体中文版.exe。
- 不修改用户原安装/下载文件；用户后来明确要求不再构建原版，参考 D:\Download\RadeonTuner-zh 的已有文件。旧baseline不要继续构建。
- 当前目标active。用户满意并明确结束后才清理源码/Git/开发环境；保留最终EXE、用户数据及运行必需文件。输出目录不能自动清空。

## 当前执行（2026-11-04）
- 用户已反馈要求继续开发，当前优先扁平UI和官方0.70合并；下面包14 EXE是旧交付，不代表新要求已完成。无清理源码/开发环境的授权，不得关闭项目。
- 唯一开发仓库即本目录，未提交改动保留。集成包13已完成：证据 .build-work/evidence/final-integration-summary-13.json。DSH会话dfbb68eb-f8b2-450a-bbfe-7f8ddb2dec1a已确认终态并永久删除。
- 窄修复包14已完成：.build-work/evidence/final-integration-summary-14.json；正式/隔离版构建0错误。启动过程中最多等待15秒唤回，不创建第二核心；实际延迟窗口4.004秒成功、已有窗口18ms、等待中退出分支通过。15秒超时分支仅静态检查。生产-debug单点改find，空参数不再误开日志，非空历史行为保持；实际表达式编译测试通过。
- 最终交付EXE为3061760字节，SHA256 61DF27CE34F2C9D1375D70BE8EB156910ED3B2523C0894F2FF0D8D9ACB5FA089；正式核心4F576AD7EC53D821C879AB20A6A666F84D54C5D83F2E1EF70054B45DF8008CAF。extract-only哈希核对通过，未普通启动正式核心/执行GPU调校/联网更新。源码diff --check通过。真实驱动功能仍需用户试用，勿声称全功能实机验收完成。
- 2026-11-04最终视觉：通过Computer Use查看隔离版调校/设置页，保持生效无卡片外重复标题，设置页顶部空操作区已消失，隐藏托盘图标开关可见，排版/深色正常。预览PID4532已关闭；DSH会话50a4a7f5-a94c-49cd-a516-c858a01c7381终态收证并永久删除。无当前DSH任务。等待用户试用/满意后才进入项目关闭清理。
- 启动器为原生静态CRT+内嵌CAB，释放LocalAppData/RadeonTuner-zh-CN/App，启动核心后退出。包13已验证5轮并发、用户配置保留、内部联接拒绝、参数转发、损坏载荷不启动及正式extract-only。66个载荷文件含官方Updater；最终输出/hash以包14为准。
- 窄审查确认：先EnsureDirectory(root)再resolve root并固定拼接App解决互斥身份随目录创建变化，锁覆盖整个RunNormal。暂勿重复全套验证。
- 设置页空操作卡片已隐藏；隔离UI七页/20弹窗/托盘/Unicode通过，尚待最终视觉核对。实际匹配函数基准192万次：原版192万次new/368640000字节，优化0次/0字节，结果一致；只代表匹配步骤，不是整程序RSS降幅。

## 已完成及证据
- 汉化覆盖XAML/提示/枚举/托盘，内部Global及配置键保持，显示映射“全局”；/utf-8；tools/verify-localization.ps1保留有意义的11项检查。
- Steam和快捷方式Unicode路径回归已修复并实际测试；快捷方式仍.url，UTF-16内容、宽路径、失败返回。
- HideTrayIcon覆盖即时切换/保存/加载/主题变更/TaskbarCreated；托盘独占HICON释放已修复。
- SingleInstanceRecall.h按引用遍历AVProcess，匹配自身EXE完整路径，当场取HWND，只存bool，无持有HANDLE对象浅复制；匹配名来自自身文件名，可改名。已真实跨进程验证。
- 后台AdlxLoopProfile及三种匹配改const引用/std::find，去掉整份vector重复复制，保持旧顺序/大小写/global/轮询间隔。独立审查确认语义相同。未卸载MainPage（后台线程持有this），未用EmptyWorkingSet。
- 深色配色/标题栏及原生菜单已实现。NativeTheme高对比恢复Default/清除dark；App处理WM_SETTINGCHANGE/WM_THEMECHANGED，有变化检测及重入保护。菜单仍CreatePopupMenu/TrackPopupMenuEx，无owner-draw。
- 保持生效外层标题及空行已删，状态勾选/警告合入卡片右侧；其他必要分组小标签保留。
- 最新正式构建 .build-work/primary-build-10.log exit0；测试构建 main-ui-validation-build-11.log exit0；localization-check-03.log PASS 11项/exit0。
- .build-work/ui-auto-02-result.json success：7页、20轮弹窗、20真实MessageBox、托盘隐藏/恢复/模拟Explorer重建、主题切换、Unicode、真实跨进程唤回全部通过，唤回约375ms。设置重载不是完整系统重启，Explorer只是模拟消息。
- 隔离UI入口跳过ADL/注册表/自启/更新及正常硬件循环。不能普通启动正式EXE做测试：正常启动会改ATI User.blb属性、同步驱动配置、写默认设置并启用调校循环。
- ui-pre/post-measurements.csv枚举准备范围不同，不能宣传精确内存降幅；约52MB只是隔离UI私有内存，不是完整GPU后台数据。后台复制减少有代码证据，仍缺有效同场景整体内存降幅。

## Codex视觉已查及待修
- 通过computer-use实际看过最新调校页/设置页/原生菜单。深色标题栏与主体正常、保持生效仅卡片内一处、状态在右、紧凑侧栏可完整访问退出；托盘隐藏开关可见。
- 原生菜单实际中文深色：设置/项目网站/退出。截图已显示在聊天。
- 设置页无内容操作区及占行已隐藏，其他页有用选择/操作保留，最终目视确认通过。
- 视觉预览PID11032及10564均已结束。勿复用旧Window ID。使用mcp__node_repl中的@oai/sky，先list_windows选唯一返回窗口；不要PowerShell UI自动化或自建截图工具。
- computer-use技能 C:\Users\78539\.codex\plugins\cache\openai-bundled\computer-use\26.1027.51404\skills\computer-use\SKILL.md 及guidance/confirmations已读。
- 预览（仅隔离版）：bin\x64\Release-UiValidation\RadeonTuner.exe -ui-stay；原生菜单 -ui-menu -ui-stay。

## 打包集成已修复（compat_review独立窄审查）
- 同目录命名互斥覆盖状态重读、解包/替换/完成标记/启动。固定resolve(root)+App保证首次创建/LocalAppData重定向的身份一致。锁寿命覆盖全部返回；5轮并发通过。
- Updater.json只缺失时创建，用户配置/Profiles/额外文件保留；程序哈希验证允许用户修改该配置。路径拒绝尾点/尾空格/保留名及App内部重解析穿越，系统LocalAppData重定向可用，实际内部联接拒绝测试通过。
- 正式载荷含未改官方Updater，启动失败保留窗口并中文提示，成功才退出；确认框提示官方更新会覆盖定制改动。未实际联网更新。
- 独立自启/开始菜单名RadeonTuner 简体中文版，右键键RadeonTunerZhCN；读写/删除成对且命令路径加引号，不碰原版身份。真实系统集成未执行。
- extract-only不启动core；宽字符参数转发/父进程退出、用户配置保留、外部更新核心保留、失败不启动均有包13/14证据。启动等待和-debug修复独立窄审查无发现。

## 官方更新器已取得（不必再找）
- 作者发布 https://github.com/dumbie/RadeonTuner/releases/download/release/RadeonTuner.zip；tag名release，published_at旧，但包内EXE是2026-10-14最新原版，勿按tag日期误判旧版。
- 下载暂存 ../official-release-check.zip；原core SHA256 D7C4487D8A183F419248FF6A9455E5C6F53532C29FB49BFD304EBD29123134DB，与用户RadeonTuner.exe.orig精确一致。只解出Updater，未运行/安装原版。
- third_party/OfficialUpdater/Updater.exe 未修改，2034688字节，SHA256 D71662119832709B367545171BDD9F9E5FA551B443C2EDF21B7F5F8F20B91385；SOURCE.md记录来源。Authenticode在此会话UnknownError，证书主体Arnold Vink，不声称签名已验证。
- 将此原组件纳入正式payload，沿原Updater.json流程保留更新。既有确认框应说明官方更新会覆盖本地汉化/界面改动，避免用户误以为更新后仍是定制版；不要删除更新功能。

## 源码/环境与清理归属
- 上游 dumbie/RadeonTuner，分支zh-cn-foundation，基线32502495deb29defc93c6152c9adb0701ad5eac2，版本0.60.0.0。C++20/C++WinRT/XAML Islands/ADL，最低Win10 18362不变。
- 上游License.md未授予修改/分发许可，已告知用户；当前仅私用交付，不上传公开发布。
- MSBuild C:\Program Files\Microsoft Visual Studio\18\BuildTools\MSBuild\Current\Bin\MSBuild.exe RadeonTuner.sln /m:2 /p:Configuration=Release /p:Platform=x64 /p:SignOutput=false；隔离版加/p:UiValidation=true，独立OutDir/IntDir/gen。访问标准SDK需require_escalated。
- BuildTools18.11.3实例98facc78、MSVC14.51.36231、SDK26100及UWP构建支持为本任务新增；补装UWP引入.NET SDK/目标包。前后清单 ../pre-build-environment.json、../post-build-environment.json 是清理依据。
- 安装前已有.NETCore6.0.36/8.0.29/10.0.6、WindowsDesktop8.0.29/10.0.6及VC++x64运行库14.51.36247；共享运行库不要盲目卸载/回退。DOTNET_TieredCompilation=0原已存在，未改。
- NuGet packages：UI.Xaml2.8.7、WebView2 1.0.4258.31、CppWinRT3.0.260818.1。../nuget.exe、../nuget-cache、../vs_buildtools.exe、下载ZIP/解压副本均本任务暂存，验收后可删。
- third_party/ArnoldVinkCode来自AeonCatalyst快照59135adaeddc337d2c20a292b2a8effb011e270f；原始快照 ../ArnoldVinkCode、ZIP可再生。ApiTokens.h可选GitHub令牌为空，无凭据。
- ../RadeonTuner-baseline是测试导出副本，不是持续开发仓库；packages为到主仓库packages的目录联接，最终清理需先正确解除联接，不能误遍历共享目标。../ui-before-memory、../ui-pre-polish是任务测试副本。
- VS安装暂存 C:\Users\78539\AppData\Local\Temp\5jiqdgfs；任务日志dd_*前缀20261102235155/235346/235422/235424与20261103000401/000402/000405/000544；dd_vs_BuildTools_decompression_log.txt为本任务。20261101013134等之前日志非本任务，不动。
- DSH各轮已结束，本地会话均已按精确路径永久清理；聊天可见消息无删除API，不能宣称已擦除。
- Codex负责设计/验收/交付，DSH实现，不用dsh_review。保留源码/测试/记录直到用户满意结束，再官方卸载任务开发环境并删除work，保留outputs及运行数据。原有/共享/归属不明组件保留并报告。
