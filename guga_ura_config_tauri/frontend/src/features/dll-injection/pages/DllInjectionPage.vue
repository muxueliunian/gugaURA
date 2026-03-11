<template>
  <div class="dll-injection-page app-page">
    <PageHeader
      eyebrow="注入链路"
      title="DLL 注入"
      description="检测游戏目录、管理 DLL 安装状态并保存注入链路配置。"
    >
      <template #actions>
        <el-button
          :loading="contextLoading"
          @click="handleRefreshCurrent"
        >
          刷新当前状态
        </el-button>
      </template>
    </PageHeader>

    <el-alert
      v-if="pageError"
      class="app-page__alert"
      type="error"
      :closable="true"
      show-icon
      title="DLL 注入页操作失败"
      :description="pageError"
      @close="handleClearError"
    />

    <div class="dll-injection-page__metrics-row">
      <InfoCard
        label="当前目录"
        :value="currentDirectoryStatus"
        :description="currentDirectoryDetail"
      />
      <InfoCard label="识别版本">
        <StatusBadge
          :label="context?.detectedVersionLabel ?? '未知版本'"
          :type="resolveGameVersionType(context?.detectedVersion ?? 'unknown')"
        />
      </InfoCard>
      <InfoCard label="安装状态">
        <StatusBadge
          :label="context?.installStatusLabel ?? '未知'"
          :type="resolveInstallStatusType(context?.installStatus ?? 'unknown')"
        />
      </InfoCard>
    </div>

    <div class="dll-injection-page__workspace">
      <SectionCard
        title="游戏目录"
        description="支持自动扫描、手动输入与选择目录，检测结果会同步更新当前注入上下文。"
      >
        <template #header-extra>
          <el-button
            text
            size="small"
            :loading="scanLoading"
            @click="handleScan"
          >
            自动扫描
          </el-button>
        </template>

        <div class="dll-injection-page__path-bar">
          <el-input
            v-model="pathInput"
            clearable
            placeholder="例如 D:\SteamLibrary\steamapps\common\..."
          />
          <el-button
            :loading="browseGameDirLoading"
            @click="handleBrowseGameDir"
          >
            选择目录
          </el-button>
          <el-button
            type="primary"
            :loading="contextLoading"
            @click="handleInspect"
          >
            检测
          </el-button>
        </div>

        <el-table
          v-if="detectedGames.length > 0"
          class="dll-injection-page__table"
          :data="detectedGames"
          row-key="path"
          size="small"
        >
          <el-table-column
            label="版本"
            width="120"
          >
            <template #default="{ row }">
              <StatusBadge
                :label="row.versionLabel"
                :type="resolveGameVersionType(row.version)"
              />
            </template>
          </el-table-column>
          <el-table-column
            prop="path"
            label="游戏目录"
            min-width="260"
            show-overflow-tooltip
          />
          <el-table-column
            label="操作"
            width="96"
            align="right"
          >
            <template #default="{ row }">
              <el-button
                link
                type="primary"
                @click="handleSelectDetectedGame(row.path)"
              >
                使用
              </el-button>
            </template>
          </el-table-column>
        </el-table>

        <PageState
          v-else-if="scanLoading"
          state="loading"
          title="正在扫描已安装游戏"
          description="等待桌面端返回当前环境中的已安装目录。"
        />

        <PageState
          v-else
          title="当前未扫描到已安装的游戏目录"
          description="可直接手动输入目录或通过“选择目录”完成检测。"
          action-text="重新扫描"
          @action="handleScan"
        />

        <el-descriptions
          v-if="context && (context.hasPath || pathInput)"
          class="dll-injection-page__inspect-result"
          :column="1"
          border
          size="small"
        >
          <el-descriptions-item label="输入路径">
            <span class="dll-injection-page__code">{{ context.path || pathInput }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="目录存在">
            <StatusBadge
              :label="context.exists ? '是' : '否'"
              :type="context.exists ? 'success' : 'warning'"
            />
          </el-descriptions-item>
          <el-descriptions-item label="有效目录">
            <StatusBadge
              :label="context.isValidGameDir ? '有效' : '无效'"
              :type="context.isValidGameDir ? 'success' : 'danger'"
            />
          </el-descriptions-item>
          <el-descriptions-item label="识别版本">
            <StatusBadge
              :label="context.detectedVersionLabel"
              :type="resolveGameVersionType(context.detectedVersion)"
            />
          </el-descriptions-item>
          <el-descriptions-item label="安装状态">
            <StatusBadge
              :label="context.installStatusLabel"
              :type="resolveInstallStatusType(context.installStatus)"
            />
          </el-descriptions-item>
        </el-descriptions>
      </SectionCard>

      <SectionCard
        title="注入链路配置"
        description="这里只保留 DLL 发给谁的注入链路参数；Receiver 监听、Relay 和 Fans 设置请在接收&转发配置页调整。"
      >
        <el-form label-position="top">
          <el-form-item
            label="Notifier 发送地址"
            :error="notifierError || undefined"
          >
            <el-input
              v-model="form.notifierHost"
              placeholder="http://127.0.0.1:4693"
            />
          </el-form-item>

          <el-form-item
            label="超时 (ms)"
            :error="timeoutError || undefined"
          >
            <el-input
              v-model="form.timeoutInput"
              placeholder="100"
            />
          </el-form-item>
        </el-form>

        <div class="dll-injection-page__action-row">
          <el-button
            type="primary"
            :disabled="Boolean(saveDisabledReason)"
            :loading="actionLoading"
            @click="handleSave"
          >
            保存配置
          </el-button>
          <el-button
            type="success"
            :disabled="Boolean(installDisabledReason)"
            :loading="actionLoading"
            @click="handleInstall"
          >
            安装 DLL
          </el-button>
          <el-button
            type="danger"
            plain
            :disabled="Boolean(uninstallDisabledReason)"
            :loading="actionLoading"
            @click="handleUninstall"
          >
            卸载 DLL
          </el-button>
        </div>

        <p class="dll-injection-page__field-hint">
          {{ actionHint }}
        </p>
      </SectionCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ElAlert } from 'element-plus/es/components/alert/index';
import { ElButton } from 'element-plus/es/components/button/index';
import { ElDescriptions, ElDescriptionsItem } from 'element-plus/es/components/descriptions/index';
import { ElForm, ElFormItem } from 'element-plus/es/components/form/index';
import { ElInput } from 'element-plus/es/components/input/index';
import { ElMessage } from 'element-plus/es/components/message/index';
import { ElTable, ElTableColumn } from 'element-plus/es/components/table/index';
import { storeToRefs } from 'pinia';
import { computed, onMounted } from 'vue';
import SectionCard from '@/components/SectionCard.vue';
import StatusBadge from '@/components/StatusBadge.vue';
import InfoCard from '@/components/display/InfoCard.vue';
import PageState from '@/components/feedback/PageState.vue';
import PageHeader from '@/components/layout/PageHeader.vue';
import { resolveGameVersionType, resolveInstallStatusType } from '@/app/utils/status';
import { useDllInjectionStore } from '@/stores/dllInjection';

defineOptions({ name: 'DllInjectionPage' });

const dllInjectionStore = useDllInjectionStore();
const {
  actionLoading,
  browseGameDirLoading,
  context,
  contextLoading,
  detectedGames,
  form,
  installDisabledReason,
  lastError,
  notifierError,
  pathInput,
  saveDisabledReason,
  scanLoading,
  timeoutError,
  uninstallDisabledReason,
} = storeToRefs(dllInjectionStore);

const pageError = computed(() => lastError.value);
const currentDirectoryStatus = computed(() => {
  if (!context.value?.path) {
    return '未选择';
  }
  return context.value.isValidGameDir ? '已确认' : '待修正';
});
const currentDirectoryDetail = computed(() => context.value?.path || '请先自动扫描或手动选择游戏目录');
const actionHint = computed(() => {
  if (saveDisabledReason.value) {
    return saveDisabledReason.value;
  }
  if (installDisabledReason.value) {
    return installDisabledReason.value;
  }
  if (uninstallDisabledReason.value) {
    return uninstallDisabledReason.value;
  }
  return '安装时会同步保存 notifier_host 与 timeout_ms；Receiver 侧参数请在接收&转发配置页保存。';
});

async function handleScan(): Promise<void> {
  await dllInjectionStore.scanGames();
}

async function handleInspect(): Promise<void> {
  await dllInjectionStore.inspectPath();
}

async function handleBrowseGameDir(): Promise<void> {
  await dllInjectionStore.browseGameDirectory();
}

async function handleSelectDetectedGame(path: string): Promise<void> {
  await dllInjectionStore.selectDetectedGame(path);
}

async function handleRefreshCurrent(): Promise<void> {
  await dllInjectionStore.refreshCurrentContext();
}

async function handleSave(): Promise<void> {
  const result = await dllInjectionStore.saveConfig();
  if (!result) {
    return;
  }

  ElMessage.success(result.notice);
}

async function handleInstall(): Promise<void> {
  const result = await dllInjectionStore.install();
  if (!result) {
    return;
  }

  if (result.notice.includes('但配置同步失败')) {
    ElMessage.warning(result.notice);
    return;
  }

  ElMessage.success(result.notice);
}

async function handleUninstall(): Promise<void> {
  const result = await dllInjectionStore.uninstall();
  if (!result) {
    return;
  }

  ElMessage.success(result.notice);
}

function handleClearError(): void {
  dllInjectionStore.clearError();
}

onMounted(() => {
  void dllInjectionStore.initialize();
  if (detectedGames.value.length === 0) {
    void dllInjectionStore.scanGames();
  }
});
</script>

<style scoped>
.dll-injection-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.dll-injection-page__metrics-row {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

.dll-injection-page__workspace {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.9fr);
  gap: 20px;
  align-items: start;
}

.dll-injection-page__path-bar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 8px;
}

.dll-injection-page__path-bar--compact {
  grid-template-columns: minmax(0, 1fr) auto;
}

.dll-injection-page__table,
.dll-injection-page__inspect-result {
  margin-top: 16px;
}

.dll-injection-page__switch-row {
  display: flex;
  gap: 12px;
  align-items: center;
  color: var(--app-text-secondary);
  font-size: 13px;
}

.dll-injection-page__action-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.dll-injection-page__field-hint {
  margin: 12px 0 0;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.dll-injection-page__code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  word-break: break-all;
}

@media (max-width: 1100px) {
  .dll-injection-page__workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .dll-injection-page__metrics-row {
    grid-template-columns: 1fr;
  }

  .dll-injection-page__path-bar,
  .dll-injection-page__path-bar--compact {
    grid-template-columns: 1fr;
  }
}
</style>
