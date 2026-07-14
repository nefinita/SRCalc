# coding: utf-8
#excel文件导入
#load_data需要添加windows和mac、linux不同的判定方式
import pandas as pd
def load_data_for_excel(file_name):
    dataset = {}
    #name = "C:/kaikedata/" + file_name
    name = file_name
    excel_file = pd.ExcelFile(name)
    dataset = excel_file.parse('Sheet1')
    return dataset