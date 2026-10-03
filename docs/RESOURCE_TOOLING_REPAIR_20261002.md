# 资源测试流程降本修复

本次针对 [性能扫描](PERFORMANCE_SCAN_20261002.md) 中用户实际观察到的测试耗时、内存和重复落盘问题。运行时、Pine 语义、公开输出 schema、冻结工作负载和资源预算均未修改；533 个核心文件哈希仍与此前资源候选一致。

## 改动

- 共享 SHA-256 工具按 1 MiB 分块读取，不再一次分配整份报告。
- Rust、Python、WASM worker 输出 `resourceReportVersion: 2` 的小型 manifest，引用旁边的完整 `.parts` 文件。原始指标数组仍在 metadata v1 中。完整 live、replica、原生 batch、seed/append、known-context 输出与全部原有比较继续执行。
- 新的报告读取器验证版本、流顺序、文件引用、端点、confirmed count 和历史控制结果。缺失任一必需副本或控制文件都会失败。
- 新增 `overheadMetrics`，将历史控制初始化/回放、replica 导出、比较、释放与报告写入从未分段时间中拆出。controller 另记录 worker wall、校验、重复比较、哈希耗时；audit 记录完整性检查和跨面比较耗时。它们与原 metrics 分开，不改变原有预算指标边界，也不声称覆盖了每个微小阶段。
- controller 每次校验只解析一次输入以统计事件，避免为四个阶段重复解析同一大输入。
- audit 对每种已校验哈希的内容完整验证 JSON，然后对相同完整字节采用快捷比较；有格式或数值差异时继续按原类型、字段、长度和数值容差比较。空文件、截断文件和非有限 JSON 数字不能通过。
- 新矩阵运行必须使用空的 surface 输出目录。恢复必须有相容收据；未知的残留报告、日志或 parts 不会被覆盖。
- 新增 [快速预检入口](RESOURCE_PRECHECK.md)：默认只运行三个 1,024-bar 单会话完整案例，各一次。完整输入、输出和原生控制均保留，结果明确标为诊断子集，不能冒称完整资源资格通过。

## 已验证的收益

| 场景 | 修复前 | 修复后 |
| --- | ---: | ---: |
| 对原来的 883,131,313 字节报告计算相同 SHA-256，Python traced allocation 峰值 | 约 842.23 MiB | 约 2.01 MiB |
| 三种 worker 的 9 个短案例主报告合计 | 6,806,838 B | 3,617 B |
| 原生 RSI default，100,000 历史加 10,000 tail，单会话主报告 | 205,086,077 B | 616 B |

完整结果留在 `.parts`，上表只衡量主报告，不把引用当作完整结果消失。9 个短案例共 27 份完整输出，以及长案例的 5 份完整输出，与对应旧候选文件逐字节相同，32/32 比较通过。长案例保留了全部 10,000 次 forming、replacement、confirmation 和原生 append 操作、完整历史控制，未缩短尾段。

哈希峰值采用两个新 Python 进程中的 tracemalloc 测量，表示 Python 分配，不是整台主机或 worker RSS。哈希耗时约 1.29 秒，未声称有显著速度提升。长案例工具经过重新编译，未用一次耗时与旧候选宣称稳定加速比。

## 验证与证据

- 原生报告的 3 项 Rust 单测通过，覆盖大输出不内嵌、控制语义和相对引用。
- 实际 WASM 的单/四流短集成回归通过，检查完整输出、样本计数、相对路径及小报告。
- 工具测试包含流式哈希内存上限、fresh worker、超时、源文件变化、未知版本、完整控制文件、禁止覆盖、相同非法 JSON 拒绝。
- audit 端到端夹具覆盖 216 条 v2 收据、540 份 live 输出和 108 次流隔离比较，并验证错误引用、错误端点、改字节、缺控制文件以及重新计算哈希后的错误值均不能通过。夹具只验证工具行为，不是新跑 216 次性能矩阵。
- Rust 结构检查通过；运行时核心文件未修改，因此未重复执行整套核心编译和完整资源矩阵。

本次结果、工具/工件身份和日志索引见 [修复凭据](../.local/test-pipeline-20261002/repair-results.json)。原始 [短案例输出比较](../.local/test-pipeline-20261002/output-comparison.json)、[长案例输出比较](../.local/test-pipeline-20261002/long-output-comparison.json)、[修复后哈希测量](../.local/test-pipeline-20261002/hash-after.json) 均保留。

## 使用和边界

本地已生成一个只重编 collector、复用已核对核心 release 库的原生工件目录。可用新目录执行默认预检：

```powershell
python scripts/resource_precheck.py --input-root .local/resource-follow-up-20261002/matrix-v14 --artifacts .local/test-pipeline-20261002/native-artifacts --output .local/resource-precheck-next
```

Python/WASM 可在核心未变时使用原有 release 工件配合当前 collector；原生 probe 需要由当前 collector 源码重建。预检会核对工件和源码身份，禁止混用变更后的核心与旧二进制。

旧版主报告是内嵌结果，新版主报告引用完整文件；消费者必须依据版本读取。当前主矩阵 auditor 支持记录中标明的 v2 manifest。旧冻结收据、旧失败结果及其原审计结论未改写；变更 collector 后不能把旧 plan 当成新测量身份直接恢复运行。新的完整资格验证需要绑定当前工具和工件。

此次没有清理旧目录的约 146.9 GiB 证据，没有实现跨 trial 内容寻址归档，也没有修改 series、map/matrix/UDT 回收等核心结构。此前 Linux Python Pivot 三项增长比失败仍然保留。快速预检不提供新的跨平台、无限会话或总体资源通过声明。
