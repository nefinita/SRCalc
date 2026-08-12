#from dataset.jsload import *
import pandas as pd
from jsload import *
import numpy as np

def Nature_Load(id,characters): #导入数据
    data = characters[characters['id'] == id]
    Name = data[0,'name']
    Hp = 0
    Atk = 0
    Def = 0
    Spd = 1
    Cr = 0
    Cd = 0
    Be = 0
    Ohb = 0
    Me = 0
    Era = 0
    Eha = 0
    Er = 0
    print(Name)
    return(Name,Hp,Atk,Def,Spd,Cr,Cd,Be,Ohb,Me,Era,Eha,Er)

def run_line_cal(characters): #计算行动点
    Spd = characters[4]
    path = 10000
    run_point = path / Spd
    return run_point

def run_line_cal_NPC():
    Spd = characters[4]
    path = 10000
    run_point = path / Spd
    return run_point

def run_line_cal_mod(id):
    mod = []
    if id in mod:
        characters = []
        Spd = characters[4]
        path = 10000
        run_point = path / Spd
        return run_point
    else:
        return 0

def run_line_code(id1,id2,id3,id4):

    light_cones,relics,characters = Load_json()
    print('请输入四个角色的id（id请查表）：')
    id1 = input('1：')
    characters1 = Nature_Load(id1,characters)
    id2 = input('2：')
    characters2 = Nature_Load(id2,characters)
    id3 = input('3：')
    characters3 = Nature_Load(id3,characters)
    id4 = input('4：')
    characters4 = Nature_Load(id4,characters)
    
    p1_point = run_line_cal(characters1)    #角色行动轴
    p2_point = run_line_cal(characters2)
    p3_point = run_line_cal(characters3)
    p4_point = run_line_cal(characters4)
    p11_point = run_line_cal_mod(id1)  #召唤物行动轴
    p22_point = run_line_cal_mod(id2)
    p33_point = run_line_cal_mod(id3)
    p44_point = run_line_cal_mod(id4)



    for i in range:
        return 0

light_cones,relics,characters = Load_json()
Nature_Load(input(),characters)