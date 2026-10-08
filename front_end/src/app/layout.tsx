import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Cuentas Claras',
  description: 'Aplicación Tauri con Next.js',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="es">
      <body>{children}</body>
    </html>
  );
}
