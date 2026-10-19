import pluginVue from 'eslint-plugin-vue'
import { defineConfigWithVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'
import eslintConfigPrettier from 'eslint-config-prettier'

export default defineConfigWithVueTs(
  {
    ignores: ['dist/**', 'src-tauri/target/**', 'src-tauri/gen/**', 'node_modules/**'],
  },
  pluginVue.configs['flat/recommended'],
  vueTsConfigs.recommended,
  eslintConfigPrettier,
  {
    rules: {
      'vue/multi-word-component-names': 'off',
      // Los dobles de prueba implementan firmas completas del contrato Bridge;
      // los parámetros no usados se marcan con prefijo '_' (p. ej. e2e-fake.ts).
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
)
