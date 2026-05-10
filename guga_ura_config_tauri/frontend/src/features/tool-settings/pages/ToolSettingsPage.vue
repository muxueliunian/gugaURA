<template>
  <div class="tool-settings-page app-page">
    <PageHeader
      eyebrow="工具偏好"
      title="工具设置"
      description="管理配置工具的开机自启，并检查是否有可用的新版本。"
    >
      <template #actions>
        <el-button
          :loading="loading"
          @click="handleRefresh"
        >
          刷新设置
        </el-button>
      </template>
    </PageHeader>

    <el-alert
      v-if="pageError"
      class="app-page__alert"
      type="error"
      :closable="true"
      show-icon
      title="工具设置操作失败"
      :description="pageError"
      @close="handleClearError"
    />

    <div class="tool-settings-page__workspace">
      <SectionCard
        title="启动设置"
        description="开启后，gugaURA 会在 Windows 登录后自动启动。"
      >
        <PageState
          v-if="loading && !context"
          state="loading"
          title="正在读取工具设置"
          description="请稍候，正在获取当前开机自启状态。"
        />

        <template v-else>
          <div class="tool-settings-page__setting-row">
            <div class="tool-settings-page__setting-copy">
              <h3>开机自启</h3>
              <p>适合需要长期使用本地 Receiver 或经常调整配置的场景。</p>
            </div>
            <el-switch
              :model-value="autostartEnabled"
              :loading="savingAutostart"
              :disabled="loading"
              active-text="开启"
              inactive-text="关闭"
              @change="handleAutostartChange"
            />
          </div>

          <el-descriptions
            class="tool-settings-page__descriptions"
            :column="1"
            border
            size="small"
          >
            <el-descriptions-item label="当前版本">
              <span class="tool-settings-page__text">{{ currentVersion || '未知' }}</span>
            </el-descriptions-item>
            <el-descriptions-item label="开机自启">
              <StatusBadge
                :label="autostartEnabled ? '已开启' : '已关闭'"
                :type="autostartEnabled ? 'success' : 'info'"
              />
            </el-descriptions-item>
          </el-descriptions>
        </template>
      </SectionCard>

      <SectionCard
        title="应用更新"
        description="手动检查 GitHub 发布页面，只提示新版本并打开下载页面。"
      >
        <PageState
          v-if="checkingUpdate && !updateResult"
          state="loading"
          title="正在检查更新"
          description="正在连接 GitHub Release 服务，请稍候。"
        />

        <template v-else>
          <div
            v-if="!updateResult"
            class="tool-settings-page__empty-update"
          >
            <PageState
              title="尚未检查更新"
              description="点击按钮后会检查当前是否有可用的新版本。"
              action-text="检查更新"
              @action="handleCheckUpdate"
            />
          </div>

          <div
            v-else
            class="tool-settings-page__update-result"
          >
            <el-alert
              :type="updateAlertType"
              :closable="false"
              show-icon
              :title="updateAlertTitle"
              :description="updateAlertDescription"
            />

            <el-descriptions
              class="tool-settings-page__descriptions"
              :column="1"
              border
              size="small"
            >
              <el-descriptions-item label="最新版本">
                <span class="tool-settings-page__text">{{ updateResult.latestVersion }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="发布时间">
                <span class="tool-settings-page__text">{{ publishedAtLabel }}</span>
              </el-descriptions-item>
              <el-descriptions-item label="更新说明">
                <p class="tool-settings-page__summary">{{ updateResult.summary }}</p>
              </el-descriptions-item>
            </el-descriptions>

            <div class="tool-settings-page__action-row">
              <el-button
                type="primary"
                :loading="checkingUpdate"
                @click="handleCheckUpdate"
              >
                重新检查
              </el-button>
              <el-button
                v-if="updateResult.hasUpdate"
                type="success"
                :loading="openingReleasePage"
                @click="handleOpenReleasePage"
              >
                打开下载页面
              </el-button>
            </div>
          </div>
        </template>
      </SectionCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ElAlert } from 'element-plus/es/components/alert/index';
import { ElButton } from 'element-plus/es/components/button/index';
import { ElDescriptions, ElDescriptionsItem } from 'element-plus/es/components/descriptions/index';
import { ElMessage } from 'element-plus/es/components/message/index';
import { ElSwitch } from 'element-plus/es/components/switch/index';
import { storeToRefs } from 'pinia';
import { computed, onMounted } from 'vue';
import SectionCard from '@/components/SectionCard.vue';
import StatusBadge from '@/components/StatusBadge.vue';
import PageState from '@/components/feedback/PageState.vue';
import PageHeader from '@/components/layout/PageHeader.vue';
import { useToolSettingsStore } from '@/stores/toolSettings';

defineOptions({ name: 'ToolSettingsPage' });

const toolSettingsStore = useToolSettingsStore();
const {
  autostartEnabled,
  checkingUpdate,
  context,
  currentVersion,
  lastError,
  loading,
  openingReleasePage,
  savingAutostart,
  updateResult,
} = storeToRefs(toolSettingsStore);

const pageError = computed(() => lastError.value);
const updateAlertType = computed(() => {
  if (updateResult.value?.versionStatus === 'ahead') {
    return 'warning';
  }

  return 'success';
});
const updateAlertTitle = computed(() => {
  if (updateResult.value?.versionStatus === 'updateAvailable') {
    return '发现可用的新版本';
  }
  if (updateResult.value?.versionStatus === 'ahead') {
    return '当前版本尚未发布';
  }
  return '当前已是最新版';
});
const updateAlertDescription = computed(() => {
  if (!updateResult.value) {
    return '';
  }

  if (updateResult.value.versionStatus === 'updateAvailable') {
    return `最新版本为 ${updateResult.value.latestVersion}，当前版本为 ${updateResult.value.currentVersion}。`;
  }
  if (updateResult.value.versionStatus === 'ahead') {
    return `当前版本 ${updateResult.value.currentVersion} 高于最新发布版本 ${updateResult.value.latestVersion}，请在发布完成后重新检查。`;
  }
  return `当前版本 ${updateResult.value.currentVersion} 已与最新发布版本一致。`;
});
const publishedAtLabel = computed(() => {
  const value = updateResult.value?.publishedAt;
  if (!value) {
    return '未提供';
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat('zh-CN', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date);
});

async function handleRefresh(): Promise<void> {
  await toolSettingsStore.loadContext();
}

async function handleAutostartChange(value: string | number | boolean): Promise<void> {
  const result = await toolSettingsStore.updateAutostart(Boolean(value));
  if (!result) {
    return;
  }

  ElMessage.success(result.notice);
}

async function handleCheckUpdate(): Promise<void> {
  const result = await toolSettingsStore.runUpdateCheck();
  if (!result) {
    return;
  }

  ElMessage.success(result.hasUpdate ? '已发现可用的新版本' : '当前已是最新版');
}

async function handleOpenReleasePage(): Promise<void> {
  const opened = await toolSettingsStore.openReleasePage();
  if (!opened) {
    return;
  }

  ElMessage.success('已打开系统浏览器');
}

function handleClearError(): void {
  toolSettingsStore.clearError();
}

onMounted(() => {
  void toolSettingsStore.initialize();
});
</script>

<style scoped>
.tool-settings-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.tool-settings-page__workspace {
  display: grid;
  grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
  gap: 20px;
  align-items: start;
}

.tool-settings-page__setting-row {
  display: flex;
  gap: 20px;
  align-items: center;
  justify-content: space-between;
}

.tool-settings-page__setting-copy {
  min-width: 0;
}

.tool-settings-page__setting-copy h3 {
  margin: 0;
  color: var(--app-text-primary);
  font-size: 14px;
  font-weight: 600;
  line-height: 1.4;
}

.tool-settings-page__setting-copy p,
.tool-settings-page__text,
.tool-settings-page__summary {
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.tool-settings-page__setting-copy p {
  margin: 6px 0 0;
}

.tool-settings-page__descriptions {
  margin-top: 16px;
}

.tool-settings-page__summary {
  margin: 0;
  white-space: pre-wrap;
}

.tool-settings-page__empty-update,
.tool-settings-page__update-result {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tool-settings-page__action-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

@media (max-width: 1100px) {
  .tool-settings-page__workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .tool-settings-page__setting-row {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
