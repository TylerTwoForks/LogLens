"use client";

import { User, Building2, LogOut, ChevronDown, Crown, Shield, Eye, UserIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import Link from "next/link";

type OrgSummary = {
  org_id: number;
  name: string;
  role: string;
};

type LicenseInfo = {
  tier: string;
  status: string;
};

type Props = {
  email: string;
  license: LicenseInfo;
  organizations: OrgSummary[];
  activeOrgId?: number | null;
};

const ROLE_ICONS: Record<string, typeof Crown> = {
  owner: Crown,
  admin: Shield,
  viewer: Eye,
};

export default function UserProfileMenu({
  email,
  license,
  organizations,
  activeOrgId,
}: Props) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="gap-2 px-3">
          <div className="flex h-7 w-7 items-center justify-center rounded-full bg-primary/10 text-primary">
            <User className="h-4 w-4" />
          </div>
          <span className="hidden text-sm text-foreground sm:inline">
            {email}
          </span>
          <ChevronDown className="h-3 w-3 text-muted-foreground" />
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="end" className="w-72">
        {/* User Section */}
        <DropdownMenuLabel className="font-normal">
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center gap-2">
              <User className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium">{email}</span>
            </div>
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="text-xs">
                {license.tier}
              </Badge>
              <Badge
                variant="outline"
                className={
                  license.status === "active"
                    ? "border-success text-success text-xs"
                    : "text-xs"
                }
              >
                {license.status}
              </Badge>
            </div>
          </div>
        </DropdownMenuLabel>

        <DropdownMenuSeparator />

        {/* Organizations Section */}
        <DropdownMenuGroup>
          <DropdownMenuLabel className="flex items-center gap-2 text-xs text-muted-foreground">
            <Building2 className="h-3 w-3" />
            Organizations
          </DropdownMenuLabel>
          {organizations.length > 0 ? (
            organizations.map((org) => {
              const RoleIcon = ROLE_ICONS[org.role] ?? UserIcon;
              const isActive = org.org_id === activeOrgId;
              return (
                <DropdownMenuItem key={org.org_id} asChild>
                  <Link
                    href={`/app?org=${org.org_id}`}
                    className={
                      isActive ? "bg-accent/10 font-medium" : ""
                    }
                  >
                    <RoleIcon className="mr-2 h-3.5 w-3.5 text-muted-foreground" />
                    <span className="flex-1 truncate">{org.name}</span>
                    <Badge variant="outline" className="ml-2 text-[10px]">
                      {org.role}
                    </Badge>
                  </Link>
                </DropdownMenuItem>
              );
            })
          ) : (
            <div className="px-2 py-1.5 text-xs text-muted-foreground">
              No organizations yet
            </div>
          )}
        </DropdownMenuGroup>

        <DropdownMenuSeparator />

        {/* Profile link */}
        <DropdownMenuItem asChild>
          <Link href="/app/profile" className="flex items-center">
            <User className="mr-2 h-4 w-4" />
            Profile
          </Link>
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        {/* Sign Out */}
        <DropdownMenuItem asChild>
          <form method="post" action="/api/auth/logout" className="w-full">
            <button
              type="submit"
              className="flex w-full items-center text-sm text-destructive"
            >
              <LogOut className="mr-2 h-4 w-4" />
              Sign out
            </button>
          </form>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
