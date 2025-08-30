#!/usr/bin/env python3
"""
测试覆盖率检查脚本
用于验证测试覆盖率是否达到目标
"""

import xml.etree.ElementTree as ET
import argparse
import sys
from pathlib import Path
from typing import Dict, Any, Optional
import json

class CoverageChecker:
    """测试覆盖率检查器"""
    
    def __init__(self, coverage_file: str = "cobertura.xml"):
        self.coverage_file = Path(coverage_file)
        self.targets = {
            "overall": 80.0,      # 整体覆盖率目标
            "line": 80.0,         # 行覆盖率目标
            "branch": 75.0,       # 分支覆盖率目标
            "function": 85.0,     # 函数覆盖率目标
        }
        
    def parse_coverage_report(self) -> Optional[Dict[str, Any]]:
        """解析覆盖率报告"""
        if not self.coverage_file.exists():
            print(f"❌ 覆盖率报告文件不存在: {self.coverage_file}")
            return None
            
        try:
            tree = ET.parse(self.coverage_file)
            root = tree.getroot()
            
            # 获取总体覆盖率
            coverage_element = root.find(".//coverage")
            if coverage_element is None:
                print("❌ 覆盖率报告中没有找到coverage元素")
                return None
                
            line_rate = float(coverage_element.get("line-rate", 0))
            branch_rate = float(coverage_element.get("branch-rate", 0))
            
            # 计算百分比
            line_coverage = line_rate * 100
            branch_coverage = branch_rate * 100
            
            # 收集各个包的覆盖率
            packages = {}
            for package in root.findall(".//package"):
                package_name = package.get("name", "unknown")
                package_line_rate = float(package.get("line-rate", 0))
                package_branch_rate = float(package.get("branch-rate", 0))
                
                packages[package_name] = {
                    "line_coverage": package_line_rate * 100,
                    "branch_coverage": package_branch_rate * 100,
                }
            
            return {
                "overall_line_coverage": line_coverage,
                "overall_branch_coverage": branch_coverage,
                "packages": packages,
                "raw": {
                    "line_rate": line_rate,
                    "branch_rate": branch_rate,
                }
            }
            
        except Exception as e:
            print(f"❌ 解析覆盖率报告失败: {e}")
            return None
    
    def check_coverage(self, coverage_data: Dict[str, Any]) -> Dict[str, Any]:
        """检查覆盖率是否达到目标"""
        results = {
            "passed": True,
            "failures": [],
            "warnings": [],
            "details": {}
        }
        
        # 检查整体行覆盖率
        line_coverage = coverage_data["overall_line_coverage"]
        results["details"]["line_coverage"] = {
            "current": line_coverage,
            "target": self.targets["line"],
            "passed": line_coverage >= self.targets["line"]
        }
        
        if line_coverage < self.targets["line"]:
            results["passed"] = False
            results["failures"].append(
                f"行覆盖率不足: {line_coverage:.1f}% < {self.targets['line']}%"
            )
        elif line_coverage < self.targets["line"] + 5:
            results["warnings"].append(
                f"行覆盖率接近阈值: {line_coverage:.1f}%"
            )
        
        # 检查整体分支覆盖率
        branch_coverage = coverage_data["overall_branch_coverage"]
        results["details"]["branch_coverage"] = {
            "current": branch_coverage,
            "target": self.targets["branch"],
            "passed": branch_coverage >= self.targets["branch"]
        }
        
        if branch_coverage < self.targets["branch"]:
            results["passed"] = False
            results["failures"].append(
                f"分支覆盖率不足: {branch_coverage:.1f}% < {self.targets['branch']}%"
            )
        elif branch_coverage < self.targets["branch"] + 5:
            results["warnings"].append(
                f"分支覆盖率接近阈值: {branch_coverage:.1f}%"
            )
        
        # 检查各个包的覆盖率
        for package_name, package_data in coverage_data["packages"].items():
            package_line_coverage = package_data["line_coverage"]
            
            # 对核心包设置更高的覆盖率要求
            if package_name.startswith("ccusage_rs::"):
                target_coverage = self.targets["function"]  # 85%
            else:
                target_coverage = self.targets["line"]     # 80%
            
            package_result = {
                "current": package_line_coverage,
                "target": target_coverage,
                "passed": package_line_coverage >= target_coverage
            }
            
            results["details"][f"package_{package_name}"] = package_result
            
            if package_line_coverage < target_coverage:
                results["warnings"].append(
                    f"包 {package_name} 覆盖率不足: {package_line_coverage:.1f}% < {target_coverage}%"
                )
        
        return results
    
    def generate_report(self, coverage_data: Dict[str, Any], 
                       check_results: Dict[str, Any]) -> str:
        """生成覆盖率报告"""
        report = []
        
        # 总体状态
        if check_results["passed"]:
            report.append("✅ 测试覆盖率检查通过")
        else:
            report.append("❌ 测试覆盖率检查失败")
        
        report.append("")
        
        # 总体覆盖率
        line_coverage = coverage_data["overall_line_coverage"]
        branch_coverage = coverage_data["overall_branch_coverage"]
        
        report.append("📊 总体覆盖率:")
        report.append(f"  行覆盖率: {line_coverage:.1f}% (目标: {self.targets['line']}%)")
        report.append(f"  分支覆盖率: {branch_coverage:.1f}% (目标: {self.targets['branch']}%)")
        report.append("")
        
        # 各个包的覆盖率
        if coverage_data["packages"]:
            report.append("📦 各包覆盖率:")
            for package_name, package_data in coverage_data["packages"].items():
                line_cov = package_data["line_coverage"]
                branch_cov = package_data["branch_coverage"]
                report.append(f"  {package_name}: {line_cov:.1f}% 行, {branch_cov:.1f}% 分支")
            report.append("")
        
        # 失败信息
        if check_results["failures"]:
            report.append("❌ 失败原因:")
            for failure in check_results["failures"]:
                report.append(f"  - {failure}")
            report.append("")
        
        # 警告信息
        if check_results["warnings"]:
            report.append("⚠️ 警告:")
            for warning in check_results["warnings"]:
                report.append(f"  - {warning}")
            report.append("")
        
        return "\n".join(report)
    
    def check_and_report(self) -> bool:
        """检查覆盖率并返回是否通过"""
        coverage_data = self.parse_coverage_report()
        if coverage_data is None:
            return False
        
        check_results = self.check_coverage(coverage_data)
        report = self.generate_report(coverage_data, check_results)
        
        print(report)
        
        return check_results["passed"]

def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="检查测试覆盖率")
    parser.add_argument("--coverage-file", default="cobertura.xml",
                       help="覆盖率报告文件路径")
    parser.add_argument("--overall-target", type=float, default=80.0,
                       help="整体覆盖率目标")
    parser.add_argument("--line-target", type=float, default=80.0,
                       help="行覆盖率目标")
    parser.add_argument("--branch-target", type=float, default=75.0,
                       help="分支覆盖率目标")
    parser.add_argument("--function-target", type=float, default=85.0,
                       help="函数覆盖率目标")
    parser.add_argument("--json", action="store_true",
                       help="输出JSON格式结果")
    
    args = parser.parse_args()
    
    # 创建检查器
    checker = CoverageChecker(args.coverage_file)
    checker.targets["overall"] = args.overall_target
    checker.targets["line"] = args.line_target
    checker.targets["branch"] = args.branch_target
    checker.targets["function"] = args.function_target
    
    # 检查覆盖率
    coverage_data = checker.parse_coverage_report()
    if coverage_data is None:
        sys.exit(1)
    
    check_results = checker.check_coverage(coverage_data)
    
    if args.json:
        # JSON输出
        result = {
            "passed": check_results["passed"],
            "coverage_data": coverage_data,
            "check_results": check_results,
            "targets": checker.targets
        }
        print(json.dumps(result, indent=2))
    else:
        # 文本输出
        report = checker.generate_report(coverage_data, check_results)
        print(report)
    
    # 返回适当的退出码
    sys.exit(0 if check_results["passed"] else 1)

if __name__ == "__main__":
    main()