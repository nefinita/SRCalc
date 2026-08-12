import json
import pandas as pd

def Load_json():
    with open("data.json") as file:
        data = json.load(file)

    light_cones = pd.DataFrame(data["light_cones"])
    print(light_cones)
    print("光锥导入完成")
    relics = pd.DataFrame(data["relics"])
    print(relics)
    print("仪器导入完成")
    characters = pd.DataFrame(data["characters"])
    print(characters)
    print("角色导入完成")
    
    return(light_cones,relics,characters)

Load_json()