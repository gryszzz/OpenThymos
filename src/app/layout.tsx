import type { ReactNode } from "react";
import type { Metadata } from "next";
import { IBM_Plex_Sans, Space_Grotesk } from "next/font/google";
import { siteConfig } from "@/lib/site";
import "./globals.css";

const bodyFont = IBM_Plex_Sans({
  subsets: ["latin"],
  variable: "--font-body",
  weight: ["400", "500", "600", "700"],
});

const displayFont = Space_Grotesk({
  subsets: ["latin"],
  variable: "--font-display",
});

export const metadata: Metadata = {
  title: `${siteConfig.name} | Governed Execution Runtime`,
  description: siteConfig.subheadline,
  icons: {
    icon: [
      { url: `${siteConfig.basePath}/favicon.ico`, sizes: "any" },
      { url: `${siteConfig.basePath}/icon.png`, type: "image/png" },
    ],
    apple: [
      { url: `${siteConfig.basePath}/apple-icon.png`, sizes: "180x180", type: "image/png" },
    ],
    shortcut: [`${siteConfig.basePath}/favicon.ico`],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${bodyFont.variable} ${displayFont.variable}`}>{children}</body>
    </html>
  );
}
