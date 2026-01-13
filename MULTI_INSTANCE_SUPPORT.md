# 多实例Channel支持功能说明

## 概述

此次更新为claude-code-notifications添加了完整的channel命名和多实例支持，允许用户配置同一类型的多个channel实例。

## 主要变更

### 1. Channel配置结构变化

**新增字段：**
- `name`: 显示名称（如"个人钉钉"、"团队协作群"）
- `channel_type`: Channel类型（system, dingtalk, feishu, wechat）

**配置示例：**
```json
{
  "channels": {
    "dingtalk_personal": {
      "name": "个人钉钉",
      "channel_type": "dingtalk",
      "enabled": true,
      "webhook_url": "..."
    },
    "dingtalk_team": {
      "name": "团队协作群",
      "channel_type": "dingtalk",
      "enabled": true,
      "webhook_url": "..."
    }
  }
}
```

### 2. UI改进

**排序规则：**
1. System channel始终在第一位
2. 其他channel按类型排序（dingtalk → feishu → wechat）
3. 同类型channel按name字母顺序排序

**新增UI元素：**
- Channel ID（只读，显示channel配置的key）
- Channel Type（只读，显示channel类型）
- Display Name（可编辑，自定义显示名称）

### 3. 后端架构调整

**ChannelManager变更：**
- 创建channel实例时从`channel_config.channel_type`读取类型
- 向后兼容：如果`channel_type`为空，使用channel_id作为类型

**Web Server变更：**
- Test endpoint使用channel_id查找配置
- 根据`channel_type`字段创建对应的channel实例

## 使用场景

### 场景1：区分个人和团队通知

```json
{
  "channels": {
    "dingtalk_personal": {
      "name": "个人钉钉",
      "channel_type": "dingtalk",
      "webhook_url": "https://oapi.dingtalk.com/robot/send?access_token=PERSONAL_TOKEN"
    },
    "dingtalk_team": {
      "name": "团队群",
      "channel_type": "dingtalk",
      "webhook_url": "https://oapi.dingtalk.com/robot/send?access_token=TEAM_TOKEN"
    }
  },
  "routing_rules": [
    {
      "name": "所有通知发个人",
      "match": {},
      "channels": ["system", "dingtalk_personal"]
    },
    {
      "name": "重要错误发团队群",
      "match": {
        "hook_types": ["Stop"],
        "message_pattern": ".*error.*"
      },
      "channels": ["dingtalk_team"]
    }
  ]
}
```

### 场景2：多团队协作

```json
{
  "channels": {
    "feishu_dev": {
      "name": "开发组",
      "channel_type": "feishu",
      "webhook_url": "https://open.feishu.cn/open-apis/bot/v2/hook/DEV_WEBHOOK"
    },
    "feishu_ops": {
      "name": "运维组",
      "channel_type": "feishu",
      "webhook_url": "https://open.feishu.cn/open-apis/bot/v2/hook/OPS_WEBHOOK"
    },
    "feishu_product": {
      "name": "产品组",
      "channel_type": "feishu",
      "webhook_url": "https://open.feishu.cn/open-apis/bot/v2/hook/PRODUCT_WEBHOOK"
    }
  }
}
```

## API变更

### Test Endpoint

**旧版本：** `POST /api/test/{channel_type}`
**新版本：** `POST /api/test/{channel_id}`

**示例：**
```bash
# 测试个人钉钉
curl -X POST http://localhost:3000/api/test/dingtalk_personal

# 测试团队群
curl -X POST http://localhost:3000/api/test/dingtalk_team
```

## 向后兼容性

- 旧配置文件无需修改，继续正常工作
- 如果`channel_type`字段为空，系统使用channel_id作为类型
- UI会正确显示旧配置的channels
- 路由规则继续使用channel_id（如"dingtalk"）

## 迁移指南

### 从旧配置迁移

**旧配置：**
```json
{
  "channels": {
    "dingtalk": {
      "enabled": true,
      "webhook_url": "..."
    }
  }
}
```

**新配置（可选迁移）：**
```json
{
  "channels": {
    "dingtalk": {
      "name": "钉钉通知",
      "channel_type": "dingtalk",
      "enabled": true,
      "webhook_url": "..."
    }
  }
}
```

### 添加新的DingTalk实例

1. 打开Web UI: `cargo run -- ui`
2. 或手动编辑`~/.claude-code-notifications.json`，添加新的channel配置：
```json
{
  "channels": {
    "dingtalk": { ... },
    "dingtalk_team": {
      "name": "团队群",
      "channel_type": "dingtalk",
      "enabled": true,
      "webhook_url": "..."
    }
  }
}
```

## 测试

所有功能已通过测试：
- ✅ 51个单元测试全部通过
- ✅ 多实例配置加载测试
- ✅ Channel创建和发送测试
- ✅ Web UI测试
- ✅ 向后兼容性测试

## 示例文件

完整的多实例配置示例：`examples/multi-instances-config.json`

包含：
- 1个system channel
- 2个dingtalk channels（个人、团队）
- 2个feishu channels（开发组、运维组）
- 1个wechat channel（告警群）
- 3个路由规则示例
