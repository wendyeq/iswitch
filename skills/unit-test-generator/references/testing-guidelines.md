# 测试生成指南

本文件为单元测试生成提供详细的技术规范和质量标准。

## Frontend 测试规范

### 环境配置
- 测试框架: Vitest + React Testing Library
- 文件位置: 与源文件同级 (Colocation)

### Header 协议
所有测试文件必须包含 FractalFlow 规范的文件头:

```typescript
/**
 * ---
 * [INPUT]:
 * - source: ./MyComponent.tsx ([POS]: /src/components/MyComponent.tsx - 目标组件)
 * [OUTPUT]: {TestSuite} - 组件测试结果
 * [POS]: /src/components/MyComponent.test.tsx - MyComponent 组件测试
 * [PROTOCOL]:
 * 1. 测试组件渲染。
 * 2. 测试用户交互。
 * 3. 测试边界条件和错误状态。
 * ---
 */
```

### 导入规范
```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@tests/test-utils';
```

### Mocking 模式

#### React Router
```typescript
vi.mock('react-router-dom', async () => ({
  ...(await vi.importActual('react-router-dom')),
  useNavigate: () => vi.fn(),
  useParams: () => ({ id: '123' }),
}));
```

#### i18n
```typescript
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
```

### 查询优先级
1. `getByRole` - 用户视角
2. `getByText`
3. `getByLabelText`
4. 避免 `.container.querySelector` 除非测试 DOM 结构

## Backend 测试规范

### 环境配置
- 测试框架: Pytest
- 文件位置: `backend/tests/unit/` 映射

### Header 协议
```python
# ---
# [INPUT]:
# - source: app/api/endpoints/users.py ([POS]: /backend/app/api/endpoints/users.py - 用户API端点)
# [OUTPUT]: {TestResult} - API端点测试结果
# [POS]: /backend/tests/unit/api/endpoints/test_users.py - 用户API测试
# [PROTOCOL]:
# 1. 测试成功场景。
# 2. 测试异常处理。
# 3. 测试权限验证。
# ---
```

### Mocking 策略
- 严禁真实 DB 调用或外部 API
- 使用 `unittest.mock.patch` 或 `pytest-mock`
- FastAPI 依赖注入使用 `app.dependency_overrides`

### Repository Mock 示例
```python
@pytest.fixture
def mock_user_repo(mocker):
    repo = mocker.MagicMock(spec=UserRepository)
    repo.get_by_id.return_value = User(id=1, name="Test")
    return repo
```

## 测试覆盖率目标

| 指标 | 目标值 |
|------|--------|
| 行覆盖率 | >= 80% |
| 必测场景 | Happy Path, Error Boundary, Loading States, Empty States |
