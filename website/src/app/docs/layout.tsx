import { DocsSidebar, DocsMobileNav } from "@/components/docs-sidebar";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="pt-24 pb-20">
      <div className="mx-auto max-w-6xl px-6 flex gap-12">
        <DocsSidebar />
        <div className="flex-1 min-w-0">
          <DocsMobileNav />
          {children}
        </div>
      </div>
    </div>
  );
}
