import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from verify_long_session_budget import evaluate


class LongSessionBudgetTests(unittest.TestCase):
    def fixture(self):
        identity = dict(historyBars=1000,tailBars=100,repetitions=2,replacementsPerBar=1,
                        sourceHash='source',binaryHash='binary',payloadHash='payload',revision='commit',
                        worktreeStatus='',formingExecutesScript=True)
        counts = dict(compile=1,historySeed=2,tailAppend=200,resultSnapshot=2,outputSerialization=2,
                      liveSeed=2,formingInitial=200,formingReplace=200,formingConfirm=200,
                      liveResultSnapshot=2,liveOutputSerialization=2)
        memory = [dict(phase=phase,repetition=repeat,source='windowsPeakWorkingSet',peakRssKiB=100,peakCommitKiB=200)
                  for repeat in range(2) for phase in ('afterHistorySeed','afterTailAppend','afterLiveSeed','afterLiveTail')]
        memory.append(dict(memory[-1],phase='afterVerification'))
        report = dict(schemaVersion=1,status='measured',**identity,realtimeExclusion=None,
                      correctness=dict(batchEqualsTail=True,repeatedHistoricalStable=True,repeatedLiveStable=True),
                      phases={k:dict(n=n,samplesMs=[1.0]*n) for k,n in counts.items()},memoryCheckpoints=memory)
        plan = dict(schemaVersion=1,expected=identity,
                    phaseBudgets={k:dict(statistic='p95' if n>=100 else 'max',limitMs=2) for k,n in counts.items()},
                    memoryBudgets=dict(source='windowsPeakWorkingSet',peakRssKiB=300,peakCommitKiB=300))
        return report,plan

    def test_complete_measurement_passes_and_cached_statistics_are_not_trusted(self):
        report,plan = self.fixture()
        report['phases']['tailAppend']['p95Ms'] = 9999
        self.assertEqual(evaluate(report,plan)['status'],'passed')

    def test_failures_and_identity_or_budget_changes_do_not_pass(self):
        report,plan = self.fixture()
        mutations = [lambda r:r.update(historyBars=10),lambda r:r.update(status='failed'),
                     lambda r:r['correctness'].update(repeatedLiveStable=False),
                     lambda r:r['phases']['tailAppend']['samplesMs'].pop(),
                     lambda r:r['phases']['tailAppend'].update(samplesMs=[3.0]*200),
                     lambda r:r['memoryCheckpoints'][0].update(peakCommitKiB=301),
                     lambda r:r['memoryCheckpoints'][0].update(peakRssKiB=None),
                     lambda r:r['memoryCheckpoints'].pop()]
        for mutate in mutations:
            broken=copy.deepcopy(report);mutate(broken)
            self.assertEqual(evaluate(broken,plan)['status'],'failed')

    def test_timeout_report_keeps_its_original_failure_reason(self):
        _,plan=self.fixture()
        result=evaluate(dict(schemaVersion=1,status='failed',error='timed out after 1800 seconds'),plan)
        self.assertEqual(result['status'],'failed')
        self.assertIn('timed out after 1800 seconds',result['failures'][0])

    def test_missing_or_nonfinite_budgets_are_rejected(self):
        report,plan=self.fixture()
        for mutate in [lambda p:p['phaseBudgets'].pop('formingConfirm'),
                       lambda p:p['expected'].pop('binaryHash'),
                       lambda p:p['phaseBudgets']['compile'].update(limitMs=float('inf'))]:
            broken=copy.deepcopy(plan);mutate(broken)
            with self.assertRaises(ValueError):evaluate(report,broken)

    def magnifier_fixture(self):
        identity = dict(historyBars=1000,tailBars=100,repetitions=2,replacementsPerBar=1,
                        sourceHash='source',binaryHash='binary',payloadHash='payload',revision='commit',
                        worktreeStatus='',formingExecutesScript=None)
        counts = dict(compile=1,historySeed=2,tailAppend=200,resultSnapshot=2,outputSerialization=2)
        memory = [dict(phase=phase,repetition=repeat,source='windowsPeakWorkingSet',peakRssKiB=100,peakCommitKiB=200)
                  for repeat in range(2) for phase in ('afterHistorySeed','afterTailAppend')]
        memory.append(dict(memory[-1],phase='afterVerification'))
        report = dict(schemaVersion=1,status='measured',qualification='notEvaluated',**identity,
                      realtimeExclusion='historical-only magnifier input',
                      correctness=dict(batchEqualsTail=True,repeatedHistoricalStable=True,repeatedLiveStable=None),
                      phases={k:dict(n=n,samplesMs=[1.0]*n) for k,n in counts.items()},memoryCheckpoints=memory)
        plan = dict(schemaVersion=1,expected=identity,realtimeExclusion='historical-only magnifier input',
                    modes=['historical','incremental'],
                    phaseBudgets={k:dict(statistic='p95' if n>=100 else 'max',limitMs=2) for k,n in counts.items()},
                    memoryBudgets=dict(source='windowsPeakWorkingSet',peakRssKiB=300,peakCommitKiB=300))
        return report,plan

    def test_magnifier_historical_only_plan_passes_without_live_phases(self):
        report,plan = self.magnifier_fixture()
        report['phases']['tailAppend']['p95Ms'] = 9999
        result = evaluate(report,plan)
        self.assertEqual(result['status'],'passed')
        self.assertEqual(result['timings']['tailAppend']['actualMs'],1.0)
        self.assertNotIn('formingConfirm', result['timings'])

    def test_magnifier_plan_rejects_live_or_partial_reports(self):
        report,plan = self.magnifier_fixture()
        live = copy.deepcopy(report)
        live['realtimeExclusion'] = None
        self.assertEqual(evaluate(live,plan)['status'],'failed')
        timeout = evaluate(dict(schemaVersion=1,status='failed',error='timed out after 1800 seconds'),plan)
        self.assertEqual(timeout['status'],'failed')
        self.assertIn('timed out after 1800 seconds', timeout['failures'][0])

    def test_freeze_recomputes_limits_from_raw_samples_before_acceptance(self):
        from verify_long_session_budget import freeze_from_report
        report,_ = self.fixture()
        report['qualification'] = 'notEvaluated'
        plan = freeze_from_report(report, name='fixture', headroom=2.0, memory_headroom=2.0)
        self.assertEqual(plan['phaseBudgets']['tailAppend']['limitMs'], 2.0)
        self.assertEqual(plan['modes'], ['historical','incremental','realtime'])
        self.assertEqual(evaluate(report,plan)['status'],'passed')
        worse = copy.deepcopy(report)
        worse['phases']['tailAppend']['samplesMs'] = [3.0]*200
        self.assertEqual(evaluate(worse,plan)['status'],'failed')


if __name__=='__main__':unittest.main()
