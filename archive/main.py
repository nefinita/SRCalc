# coding: utf-8
# main主文件
import os
from dataset.jsload import *
from dataset.excel import *
from dataset.dataload import *
from calcu.calculator import *

def clear_screen():
    # 判断系统类型
    if os.name == 'nt':  # Windows系统
        os.system('cls')
    else:  # Mac和Linux系统
        os.system('clear')

def choose(tool):
    if tool == "1":
        print('四角色排轴计算器')

    elif tool == "2":
        print('角色配装优化器')
        light_cones,relics,characters = Load_json()
        
    elif tool == 'q':
        print('退出')
        return 1
    else:
        print('未找到组件，请重新输入，如有问题请输入q退出')
        main()

def main():
    #主函数
    print("欢迎使用星铁排轴计算小助手")
    print("如果输错了直接关掉重开")
    print('输入数字即可使用指定工具')
    print('输入q即可退出')
    print("1 四角色排轴计算器   2 角色配装优化器")  
    tool = input("请输入您需要使用的工具：")
    choose(tool)
main()