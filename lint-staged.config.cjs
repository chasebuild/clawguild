const quote = (value) => JSON.stringify(value);

const buildNextLintCommand = (files) => {
  const dashboardFiles = files
    .filter((file) => file.startsWith('dashboard/'))
    .map((file) => file.replace(/^dashboard\//, ''));

  if (dashboardFiles.length === 0) {
    return [];
  }

  const fileArgs = dashboardFiles.map((file) => `--file ${quote(file)}`).join(' ');
  return [`pnpm -C dashboard exec next lint ${fileArgs}`];
};

module.exports = {
  'dashboard/**/*.{ts,tsx}': (files) => {
    const commands = buildNextLintCommand(files);
    commands.push(`prettier --write ${files.map(quote).join(' ')}`);
    return commands;
  },

  '**/*.{ts,tsx}': (files) => {
    const nonDashboardFiles = files.filter((file) => !file.startsWith('dashboard/'));
    if (nonDashboardFiles.length === 0) {
      return [];
    }
    return [`prettier --write ${nonDashboardFiles.map(quote).join(' ')}`];
  },

  '**/*.rs': (files) => {
    const commands = [];

    if (files.length > 0) {
      commands.push(`rustfmt --edition 2021 ${files.map(quote).join(' ')}`);
    }

    commands.push('cargo clippy --workspace --all-targets -- -D warnings');
    return commands;
  },
};
