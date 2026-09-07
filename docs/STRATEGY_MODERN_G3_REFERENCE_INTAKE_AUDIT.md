# G3 独立参考输入接收审计

日期：2026-09-07。状态：partial；G3 可比场景验收仍 blocked。

用户授权检查 `I:\sys\下载`，对应 WSL `/mnt/i/sys/下载`。
已保存 8 份 18g TradingView Tester 成交 CSV，以及 1 份 ADAUSDT.P 的一分钟 K 线 CSV。
原始文件和逐文件 SHA-256 清单位于
`.local/five-stage-evidence/stage3-download-intake/manifest.json`，未放入公共 fixture。

## 文件核验

- 8 份成交 CSV 只有 6 份内容唯一：ENTRY_first 与 EXIT_first 完全相同；
  capacity 基础文件与 `(1)` 完全相同。文件名不同不能算成独立观察。
- 按唯一内容统计有 6 个已平仓交易组，以及 3 个未平仓/不完整交易组。
- 中文列包含交易编号、类型、日期和时间、信号、价格、数量、净损益和手续费。
  `开盘价`、`—` 出现在未平仓记录中，不能转换成实际平仓时间或零价格。
- K 线共 1532 行；`time` 使用 epoch 秒，范围 1788345840–1788437700。
  成交 CSV 的本地时间字符串尚未确认时区，不能直接映射 bar 索引。

目前没有找到与各份导出一一对应的 Pine 源码、导入库、参数、strategy 设置和精确
回测/预热范围；旧审计提到的 `18g-b1-evidence-package.zip` 与
`stage18g-source-free-runs.json` 也未找到。不能反推一份“结果相似”的源码充当原始输入。
这些文件可以保留为参考来源，但尚不能计算解释器一致率。

## Chrome 尝试

用户进一步授权尝试 Chrome 插件控制。当前任务没有暴露 Chrome/browser 控制工具，
插件发现/安装能力也未提供可调用的 Chrome 入口。未声称插件已安装或已连接用户浏览器。

通过本地 Linux Chrome 的独立临时 profile 和 CDP 成功打开 TradingView 图表及 Pine
编辑器；点击 Add to chart 出现 Sign in。此会话未登录，无法执行新策略获得独立结果。
Windows 程序启动另返回格式错误；没有连接到用户已登录的 Windows Chrome 会话。
没有生成新 Tester 参考，也没有访问或提取登录凭据。

恢复条件是取得匹配原始输入包，或在已认证的 TradingView 浏览器中重新运行并冻结
一份可复现策略、设置、K 线与导出。随后才能映射字段、逐笔比较并决定是否修改成交语义。
性能 A/B 和解释器内部回归不能代替该步骤。
