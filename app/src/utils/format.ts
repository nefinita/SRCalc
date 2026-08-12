export function formatNumber(n: number): string {
  if (n >= 1e8) return (n / 1e8).toFixed(2) + " 亿";
  if (n >= 1e4) return (n / 1e4).toFixed(1) + " 万";
  return Math.round(n).toLocaleString("zh-CN");
}

export function formatPercent(n: number): string {
  return (n * 100).toFixed(1) + "%";
}
