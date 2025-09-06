use crate::config::{ProcessConfig, RealmConfig};
use crate::templates::template::{Template, TemplateFile};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn create_template(templates_dir: &Path) -> Result<()> {
    let template_dir = templates_dir.join("nextjs");
    if template_dir.exists() {
        return Ok(());
    }

    fs::create_dir_all(&template_dir)?;

    let files = vec![
        TemplateFile {
            path: "package.json".to_string(),
            content: r#"{
  "name": "nextjs-app",
  "type": "module",
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  },
  "dependencies": {
    "next": "14.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/react": "^18.0.0",
    "@types/react-dom": "^18.0.0",
    "typescript": "^5.0.0"
  }
}"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "next.config.js".to_string(),
            content: r#"/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
}

module.exports = nextConfig
"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "app/layout.tsx".to_string(),
            content: r#"export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "app/page.tsx".to_string(),
            content: r#"'use client';

import { useState, useEffect } from 'react';

export default function Home() {
  const [message, setMessage] = useState('Loading...');

  useEffect(() => {
    fetch('/api/health')
      .then(res => res.json())
      .then(data => setMessage(`API Status: ${data.status}`))
      .catch(() => setMessage('API connection failed'));
  }, []);

  return (
    <div style={{ padding: '2rem', textAlign: 'center' }}>
      <h1>Next.js Full-Stack App</h1>
      <p>{message}</p>
    </div>
  );
}
"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "app/api/health/route.ts".to_string(),
            content: r#"import { NextResponse } from 'next/server';

export async function GET() {
  return NextResponse.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    service: 'nextjs'
  });
}
"#
            .to_string(),
            executable: false,
        },
    ];

    let realm_config = RealmConfig {
        env: HashMap::new(),
        env_file: Some(".env.local".to_string()),
        processes: {
            let mut processes = HashMap::new();
            processes.insert(
                "nextjs".to_string(),
                ProcessConfig {
                    command: "bun run dev".to_string(),
                    port: Some(3000),
                    routes: vec!["/".to_string()],
                    working_directory: None,
                },
            );
            processes
        },
        proxy_port: 8000,
    };

    let template = Template {
        name: "nextjs".to_string(),
        description: "Next.js 14 full-stack application with App Router".to_string(),
        version: "1.0.0".to_string(),
        files,
        realm_config,
        variables: HashMap::new(),
    };

    let template_content = serde_yaml::to_string(&template)?;
    fs::write(template_dir.join("template.yml"), template_content)?;

    Ok(())
}
