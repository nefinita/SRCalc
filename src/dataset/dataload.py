import torch
import pandas
from dataset.jsload import *
from dataset.excel import *

def LoadData():
    light_cones = load_data_for_excel("light_cones.xlsx") #光锥
    print("光锥导入完成")
    relics = load_data_for_excel("relics.xlsx") #仪器
    print("仪器导入完成")
    characters = load_data_for_excel("characters.xlsx") #角色
    print("角色导入完成")
    return light_cones,relics,characters