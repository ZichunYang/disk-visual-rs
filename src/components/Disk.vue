<script setup>
import { ClearOutlined, CopyOutlined, DeleteOutlined, FolderOpenOutlined } from "@ant-design/icons-vue";
</script>
<template>
    <a-modal v-model:open="open" :closable="closable" :footer="null" :width="modalWidth" @ok="(e) => { console.log(e) }">
        <a-input id="targetFolder" :bordered="false" placeholder="目标文件夹" size="large" v-model:value="value"
            @change="getRecommendFolders" @pressEnter="startScanFolder" />
        <div v-for="suggestion in suggestions" style="padding: 11px;">
            <span style="color: grey">{{ suggestion }}</span>
        </div>
    </a-modal>
    <div id="main">
    </div>
    <div v-if="selectedFolderName !== ''" :style="{ position: 'absolute', right: '0px', bottom: '70%', width: '200px' }">
        <div style="background-color: #fff; border-radius: 5px; padding: 4px;">
            <div v-for="name in selectedFolderName">
                <a-tag color="#55acee">
                    <template #icon>
                      <FolderOpenOutlined />
                    </template>
                    {{ name }}
                </a-tag>
            </div>
        </div>
    </div>
    <a-float-button-group shape="square" :style="{ right: '160px', bottom: '50%' }">
        <a-float-button @click="copy">
            <template #icon>
                <CopyOutlined />
            </template>
        </a-float-button>
        <a-float-button>
            <template #icon>
                <a-popconfirm title="Are you sure delete this  file or folder?" ok-text="Yes" cancel-text="No"
                    @confirm="deleteByPath">
                    <DeleteOutlined />
                </a-popconfirm>
            </template>
        </a-float-button>
        <a-float-button @click="clear">
            <template #icon>
                <ClearOutlined />
            </template>
        </a-float-button>
    </a-float-button-group>
</template>

<script>
import { invoke } from "@tauri-apps/api/tauri";
import { message } from "ant-design-vue";
import { TreemapChart } from 'echarts/charts';
import { TitleComponent, TooltipComponent } from 'echarts/components';
import * as echarts from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { ref } from "vue";

const open = ref(true);
const closable = ref(false);
const modalWidth = ref('80%');
const value = ref('');
const interval = ref(null);
const os = ref('');
const suggestions = ref([]);
const selectedFolder = ref('');
const selectedFolderName = ref([]);

echarts.use([TitleComponent, TooltipComponent, TreemapChart, CanvasRenderer]);

export default {
    data() {
        return {
            open,
            closable,
            modalWidth,
            value,
            interval,
            os,
            suggestions,
            selectedFolder,
            selectedFolderName,
        }
    },
    methods: {
        async getRecommendFolders() {
            await invoke('get_recommend_folders', { currentPath: value.value }).then((recommendFolders) => {
                suggestions.value = recommendFolders.slice(0, 10);
            });
        },
        async startScanFolder() {
            this.$options.methods.resizeMainDom();
            await invoke('start_scan_folder', { path: value.value });
            open.value = false;
            await this.$options.methods.renderDisk();
            interval.value = setInterval(async () => {
                await invoke('is_scanning', {}).then(async (isScanning) => {
                    if (isScanning) {
                        await this.$options.methods.updateDisk();
                    } else {
                        await this.$options.methods.updateDisk();
                        clearInterval(interval.value);
                    }
                });
            }, 1000);
        },
        resizeMainDom() {
            let mainDiv = document.getElementById('main');

            const windowWidth = window.innerWidth;
            const windowHeight = window.innerHeight;

            mainDiv.style.width = windowWidth * 0.8 + 'px';
            mainDiv.style.height = windowHeight * 0.8 + 'px';

            console.log("Width in pixels: " + mainDiv.offsetWidth);
            console.log("Height in pixels: " + mainDiv.offsetHeight);
        },
        async renderDisk() {
            await invoke('get_current_os', {}).then((value) => {
                os.value = value;
            });
            let chartDom = document.getElementById('main');
            chartDom.hidden = false;
            let myChart = echarts.init(chartDom);
            let option;
            let basePath = value.value;
            let separator = os.value === 'Windows' ? '\\' : '/';
            if (basePath[basePath.length - 1] !== separator) {
                basePath += separator;
            }

            myChart.showLoading();
            myChart.hideLoading();

            function formatFileSize(byte) {
                let baseUnit = 1024.0;

                if (os.value === 'Windows') {
                    baseUnit = 1024.0;
                } else if (
                    os.value === 'MacOS' ||
                    os.value === 'Linux'
                ) {
                    baseUnit = 1000.0;
                }

                if (byte < baseUnit) {
                    return `${byte} B`;
                }

                const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
                let size = byte;
                let reducedSize = size;
                let unitIdx = 0;

                while (size >= baseUnit && unitIdx < units.length - 1) {
                    reducedSize = size;
                    size /= baseUnit;
                    unitIdx++;
                }

                if (size < 1.0 && unitIdx > 0) {
                    unitIdx--;
                    size = reducedSize;
                }

                return `${size.toFixed(2)} ${units[unitIdx]}`;
            }

            function getLevelOption() {
                return [
                    {
                        itemStyle: {
                            borderColor: '#999',
                            borderWidth: 5,
                            gapWidth: 1
                        },
                        upperLabel: {
                            show: true,
                            color: '#fff',
                        },
                    },
                    {
                        colorSaturation: [0.35, 0.5],
                        itemStyle: {
                            borderWidth: 5,
                            gapWidth: 1,
                            borderColorSaturation: 0.5
                        },
                        upperLabel: {
                            show: true,
                            color: '#fff',
                        },
                    },
                    {
                        colorSaturation: [0.35, 0.5],
                        itemStyle: {
                            borderWidth: 5,
                            gapWidth: 1,
                            borderColorSaturation: 0.4
                        },
                        upperLabel: {
                            show: true,
                            color: '#fff',
                        },
                    },
                    {
                        colorSaturation: [0.35, 0.5],
                        itemStyle: {
                            borderWidth: 5,
                            gapWidth: 1,
                            borderColorSaturation: 0.3
                        },
                        upperLabel: {
                            show: true,
                            color: '#fff',
                        },
                    },
                    {
                        colorSaturation: [0.35, 0.5],
                        itemStyle: {
                            borderWidth: 5,
                            gapWidth: 1,
                            borderColorSaturation: 0.2
                        },
                        upperLabel: {
                            show: true,
                            color: '#fff',
                        },
                    },
                    {
                        colorSaturation: [0.35, 0.5],
                        itemStyle: {
                            borderWidth: 5,
                            gapWidth: 1,
                            borderColorSaturation: 0.1
                        },
                        upperLabel: {
                            show: true,
                            color: '#fff',
                        },
                    },
                ];
            }

            myChart.setOption(
                (option = {
                    title: {
                        text: 'Disk Usage',
                        left: 'center'
                    },
                    tooltip: {
                        formatter: function (info) {
                            let value = info.value;
                            let treePathInfo = info.treePathInfo;
                            let treePath = [];
                            for (let i = 1; i < treePathInfo.length; i++) {
                                treePath.push(treePathInfo[i].name);
                            }
                            let relativePath = echarts.format.encodeHTML(treePath.join(separator));
                            return [
                                '<div class="tooltip-title">' +
                                (relativePath === '' ? basePath.substring(0, basePath.length - 1) : basePath + relativePath) +
                                '</div>',
                                'Disk Usage: ' + formatFileSize(value),
                            ].join('');
                        }
                    },
                    series: [
                        {
                            name: value.value,
                            type: 'treemap',
                            visibleMin: 300,
                            label: {
                                show: true,
                                formatter: function (params) {
                                    return params.data.name + ' (' + formatFileSize(params.data.value) + ')';
                                }
                            },
                            upperLabel: {
                                show: true,
                                height: 30
                            },
                            itemStyle: {
                                borderColor: '#fff'
                            },
                            levels: getLevelOption(),
                            data: await invoke('get_folder_info', {}),
                            roam: false,
                        }
                    ]
                })
            );
            myChart.on('click', function (params) {
                selectedFolder.value = ''
                for (let i = 0; i < params.treePathInfo.length; i++) {
                    let name = params.treePathInfo[i].name;
                    // end with separator
                    if (name[name.length - 1] === separator) {
                        selectedFolder.value += name;
                    } else {
                        selectedFolder.value += name + separator;
                    }
                    selectedFolderName.value = name.split(separator).filter((item) => item !== '');
                }
                selectedFolderName.value = [selectedFolderName.value[selectedFolderName.value.length - 1]]
                selectedFolder.value = selectedFolder.value.substring(0, selectedFolder.value.length - 1);
            });
        },
        async updateDisk() {
            let chartDom = echarts.getInstanceByDom(document.getElementById('main'));
            let option = chartDom.getOption();
            option.series[0].data = await invoke('get_folder_info', {});
            chartDom.setOption(option);
        },
        async clear() {
            await invoke('stop_scan_folder_and_clear', {}).then(
                () => {
                    open.value = true;
                    value.value = '';
                    suggestions.value = [];
                    clearInterval(interval.value);
                    let chartDom = echarts.getInstanceByDom(document.getElementById('main'));
                    chartDom.clear();
                    selectedFolder.value = '';
                    selectedFolderName.value = [];
                }
            )
            document.getElementById('targetFolder').focus();
        },
        copy() {
            if (selectedFolder.value === '') {
                message.error('Please select a folder');
                return;
            }
            navigator.clipboard.writeText(selectedFolder.value).then(() => {
                message.success(selectedFolder.value + ' copied to clipboard');
            }, () => {
                message.error('Failed to copy ' + selectedFolder.value + ' to clipboard');
            });
        },
        deleteByPath() {
            if (selectedFolder.value === '') {
                message.error('Please select a folder');
                return;
            }
            invoke('delete_path', { path: selectedFolder.value }).then(() => {
                message.success(selectedFolder.value + ' deleted');
            }, () => {
                message.error('Failed to delete ' + selectedFolder.value);
            });
        }
    },
    mounted() {
        document.getElementById('targetFolder').focus();
    }
}
</script>
