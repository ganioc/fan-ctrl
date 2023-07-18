#!/bin/bash

echo "fan ctrl"

I2C_DEV=1
I2C_ADDR=0x4C
REG_ADC=0x4C

SLEEP_SECONDS=10

STAGE1=1
STAGE2=2
STAGE3=3
STAGE4=4

# in 0.001 degree
TEMP_DELTA=2000
TEMP1=40000
TEMP2=48000
TEMP3=56000

STAGE1_UP_TEMP=$(expr ${TEMP1} + ${TEMP_DELTA})
STAGE2_UP_TEMP=$(expr ${TEMP2} + ${TEMP_DELTA})
STAGE2_DN_TEMP=$(expr ${TEMP1} - ${TEMP_DELTA})
STAGE3_UP_TEMP=$(expr ${TEMP3} + ${TEMP_DELTA})
STAGE3_DN_TEMP=$(expr ${TEMP2} - ${TEMP_DELTA})
STAGE4_DN_TEMP=$(expr ${TEMP3} - ${TEMP_DELTA})

# echo "STAGE1_UP_TEMP: ${STAGE1_UP_TEMP}"

CTRL_ADC1=16
CTRL_ADC2=32
CTRL_ADC3=48
CTRL_ADC4=63

stage=${STAGE2}
temp_stage=CTRL_ADC$stage 
# echo "temp stage: ${!temp_stage}"

i2cset -y ${I2C_DEV} ${I2C_ADDR} ${REG_ADC} ${!temp_stage}
sleep ${SLEEP_SECONDS} 


while true; do
    echo "stage: ${stage}"
    # get cpu temp
    CURRENT_TEMP=$(cat /sys/class/thermal/thermal_zone0/temp)
    echo ${CURRENT_TEMP}

    case ${stage} in
        1)
            if [ ${CURRENT_TEMP} -ge ${STAGE1_UP_TEMP} ]; then 
                stage=2
            fi
            ;;
        2)
            if [ ${CURRENT_TEMP} -ge ${STAGE2_UP_TEMP} ]; then 
                stage=3
            elif [ ${CURRENT_TEMP} -le ${STAGE2_DN_TEMP} ]; then 
                stage=1
            fi
            ;;
        3)
            if [ ${CURRENT_TEMP} -ge ${STAGE3_UP_TEMP} ]; then 
                stage=4
            elif [ ${CURRENT_TEMP} -le ${STAGE3_DN_TEMP} ]; then 
                stage=2
            fi
            ;;
        4)
            if [ ${CURRENT_TEMP} -le ${STAGE4_DN_TEMP} ]; then 
                stage=3
            fi
            ;;
        *)
            echo "stage error"

    esac
    
    temp_stage=CTRL_ADC$stage 
    # echo "temp stage: ${!temp_stage}"

    i2cset -y ${I2C_DEV} ${I2C_ADDR} ${REG_ADC} ${!temp_stage}

    sleep ${SLEEP_SECONDS}

done


