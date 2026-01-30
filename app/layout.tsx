// import type { Metadata } from "next";
// import { Geist_Mono, Inter } from "next/font/google";
// import "./globals.css";
// import { Providers } from "./components/providers";

// const inter = Inter({
//   variable: "--font-inter",
//   subsets: ["latin"],
//   display: "swap",
// });

// const geistMono = Geist_Mono({
//   variable: "--font-geist-mono",
//   subsets: ["latin"],
// });

// export const metadata: Metadata = {
//   title: "Solana dApp Starter",
//   description: "A minimal Next.js starter powered by @solana/react-hooks",
//   icons: {
//     icon: "/icon.svg",
//     shortcut: "/icon.svg",
//     apple: "/icon.svg",
//   },
// };

// export default function RootLayout({
//   children,
// }: Readonly<{
//   children: React.ReactNode;
// }>) {
//   return (
//     <html lang="en">
//       <Providers>
//         <body
//           suppressHydrationWarning
//           className={`${inter.variable} ${geistMono.variable} antialiased`}
//         >
//           {children}
//         </body>
//       </Providers>
//     </html>
//   );
// }



import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import AppWalletProvider from "./components/AppWalletProvider"; // <--- Import the new provider

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Solana Gig Board",
  description: "Decentralized Bounty Platform",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        {/* Wrap everything inside the Provider */}
        <AppWalletProvider>
          {children}
        </AppWalletProvider>
      </body>
    </html>
  );
}