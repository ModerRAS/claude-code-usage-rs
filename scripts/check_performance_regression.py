#!/usr/bin/env python3
"""
性能回归检查脚本
用于比较基准测试结果，检测性能回归
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, Any, List, Optional
import argparse

class PerformanceRegressionChecker:
    """性能回归检查器"""
    
    def __init__(self, criterion_dir: str = "target/criterion"):
        self.criterion_dir = Path(criterion_dir)
        self.thresholds = {
            "time": 0.10,  # 10%的时间回归阈值
            "memory": 0.15,  # 15%的内存回归阈值
        }
        
    def load_benchmark_results(self, benchmark_name: str) -> Dict[str, Any]:
        """加载基准测试结果"""
        benchmark_dir = self.criterion_dir / benchmark_name / "new"
        if not benchmark_dir.exists():
            return {}
            
        # 查找estimates.json文件
        estimates_file = benchmark_dir / "estimates.json"
        if not estimates_file.exists():
            return {}
            
        try:
            with open(estimates_file, 'r') as f:
                return json.load(f)
        except (json.JSONDecodeError, IOError):
            return {}
    
    def compare_benchmarks(self, baseline_results: Dict[str, Any], 
                         current_results: Dict[str, Any]) -> Dict[str, Any]:
        """比较基准测试结果"""
        comparison = {
            "time_regression": False,
            "memory_regression": False,
            "time_change": 0.0,
            "memory_change": 0.0,
            "details": {}
        }
        
        # 比较执行时间
        if "Mean" in baseline_results and "Mean" in current_results:
            baseline_time = baseline_results["Mean"]["point_estimate"]
            current_time = current_results["Mean"]["point_estimate"]
            
            if baseline_time > 0:
                time_change = (current_time - baseline_time) / baseline_time
                comparison["time_change"] = time_change
                comparison["time_regression"] = time_change > self.thresholds["time"]
                
                comparison["details"]["time"] = {
                    "baseline": baseline_time,
                    "current": current_time,
                    "change_percent": time_change * 100,
                    "threshold": self.thresholds["time"] * 100
                }
        
        # 比较内存使用（如果可用）
        if "memory" in baseline_results and "memory" in current_results:
            baseline_memory = baseline_results["memory"]["point_estimate"]
            current_memory = current_results["memory"]["point_estimate"]
            
            if baseline_memory > 0:
                memory_change = (current_memory - baseline_memory) / baseline_memory
                comparison["memory_change"] = memory_change
                comparison["memory_regression"] = memory_change > self.thresholds["memory"]
                
                comparison["details"]["memory"] = {
                    "baseline": baseline_memory,
                    "current": current_memory,
                    "change_percent": memory_change * 100,
                    "threshold": self.thresholds["memory"] * 100
                }
        
        return comparison
    
    def check_regressions(self) -> List[Dict[str, Any]]:
        """检查所有基准测试的回归"""
        regressions = []
        
        # 获取所有基准测试目录
        benchmark_dirs = [d for d in self.criterion_dir.iterdir() 
                         if d.is_dir() and d.name != "base"]
        
        for benchmark_dir in benchmark_dirs:
            benchmark_name = benchmark_dir.name
            
            # 加载基线结果
            baseline_results = self.load_benchmark_results(f"{benchmark_name}/base")
            if not baseline_results:
                continue
                
            # 加载当前结果
            current_results = self.load_benchmark_results(benchmark_name)
            if not current_results:
                continue
                
            # 比较结果
            comparison = self.compare_benchmarks(baseline_results, current_results)
            
            # 如果有回归，添加到结果列表
            if comparison["time_regression"] or comparison["memory_regression"]:
                regressions.append({
                    "benchmark": benchmark_name,
                    "comparison": comparison
                })
        
        return regressions
    
    def generate_report(self, regressions: List[Dict[str, Any]]) -> str:
        """生成回归报告"""
        if not regressions:
            return "✅ 没有检测到性能回归"
        
        report = ["⚠️ 检测到性能回归：", ""]
        
        for regression in regressions:
            benchmark_name = regression["benchmark"]
            comparison = regression["comparison"]
            
            report.append(f"📊 {benchmark_name}:")
            
            if comparison["time_regression"]:
                time_details = comparison["details"]["time"]
                report.append(f"  ⏱️  时间回归: +{time_details['change_percent']:.2f}% "
                             f"(阈值: {time_details['threshold']:.1f}%)")
            
            if comparison["memory_regression"]:
                memory_details = comparison["details"]["memory"]
                report.append(f"  💾 内存回归: +{memory_details['change_percent']:.2f}% "
                             f"(阈值: {memory_details['threshold']:.1f}%)")
            
            report.append("")
        
        return "\n".join(report)
    
    def check_and_report(self) -> bool:
        """检查回归并返回是否通过"""
        regressions = self.check_regressions()
        report = self.generate_report(regressions)
        
        print(report)
        
        # 如果有回归，返回失败
        return len(regressions) == 0

def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="检查性能回归")
    parser.add_argument("--criterion-dir", default="target/criterion",
                       help="基准测试结果目录")
    parser.add_argument("--time-threshold", type=float, default=0.10,
                       help="时间回归阈值 (默认: 0.10 = 10%)")
    parser.add_argument("--memory-threshold", type=float, default=0.15,
                       help="内存回归阈值 (默认: 0.15 = 15%)")
    parser.add_argument("--json", action="store_true",
                       help="输出JSON格式结果")
    
    args = parser.parse_args()
    
    # 创建检查器
    checker = PerformanceRegressionChecker(args.criterion_dir)
    checker.thresholds["time"] = args.time_threshold
    checker.thresholds["memory"] = args.memory_threshold
    
    # 检查回归
    regressions = checker.check_regressions()
    
    if args.json:
        # JSON输出
        result = {
            "has_regressions": len(regressions) > 0,
            "regressions": regressions,
            "thresholds": checker.thresholds
        }
        print(json.dumps(result, indent=2))
    else:
        # 文本输出
        report = checker.generate_report(regressions)
        print(report)
    
    # 返回适当的退出码
    sys.exit(0 if len(regressions) == 0 else 1)

if __name__ == "__main__":
    main()