"use client"

import {
  BadgeCheck,
  ChevronsUpDown,
  LogOut,
  LogIn,
  UserPlus,
} from "lucide-react"

import { Avatar, AvatarFallback } from "@/components/ui/avatar"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar"
import { useClient } from "@/hooks/useClient"
import { AuthDialog } from "./AuthDialog"
import { useState } from "react"

export function NavUser() {
  const { userData: user, isLoggedIn, logout } = useClient()
  const { isMobile } = useSidebar()
  const [showLoginDialog, setShowLoginDialog] = useState(false)
  const [showRegisterDialog, setShowRegisterDialog] = useState(false)

  if (!isLoggedIn) {
    return (
      <>
        <AuthDialog
          defaultTab="login"
          isOpen={showLoginDialog}
          onOpenChange={setShowLoginDialog}
        />
        <AuthDialog
          defaultTab="register"
          isOpen={showRegisterDialog}
          onOpenChange={setShowRegisterDialog}
        />
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton
                  size="lg"
                  className="rounded-full data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground">
                  <Avatar className="w-8 h-8 rounded-lg ">
                    <AvatarFallback className="rounded-lg bg-zinc-700">
                      ?
                    </AvatarFallback>
                  </Avatar>
                  <div className="grid flex-1 text-sm leading-tight text-left">
                    <span className="font-semibold truncate">Guest</span>
                    <span className="text-xs truncate text-muted-foreground">
                      Not signed in
                    </span>
                  </div>
                  <ChevronsUpDown className="ml-auto size-4" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                className="w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg"
                side={isMobile ? "bottom" : "right"}
                align="end"
                sideOffset={4}>
                <DropdownMenuLabel className="text-center">
                  Welcome to Organ Pages
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuGroup>
                  <DropdownMenuItem onSelect={() => setShowLoginDialog(true)}>
                    <LogIn className="mr-2 size-4" />
                    Sign In
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onSelect={() => setShowRegisterDialog(true)}>
                    <UserPlus className="mr-2 size-4" />
                    Create Account
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
      </>
    )
  }

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="rounded-xl data-[state=open]:bg-zinc-900 data-[state=open]:text-zinc-100 hover:bg-zinc-900 hover:text-zinc-100">
              <Avatar className="w-8 h-8 rounded-lg ">
                {/* <AvatarImage src={user?.avatar} alt={user?.name} /> */}
                <AvatarFallback className="rounded-lg bg-zinc-700">
                  {user?.metadata?.name?.charAt(0)}
                </AvatarFallback>
              </Avatar>
              <div className="grid flex-1 text-sm leading-tight text-left">
                <span className="font-semibold truncate">
                  Untitled Project{/* {user?.metadata?.name} */}
                </span>
                <span className="text-xs truncate">{user?.email}</span>
              </div>
              <ChevronsUpDown className="ml-auto size-4" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg"
            side={isMobile ? "bottom" : "right"}
            align="end"
            sideOffset={4}>
            <DropdownMenuLabel className="p-0 font-normal">
              <div className="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                <Avatar className="w-8 h-8 rounded-lg">
                  {/* <AvatarImage src={user.avatar} alt={user.name} /> */}
                  <AvatarFallback className="rounded-lg">
                    {user?.metadata?.name?.charAt(0)}
                  </AvatarFallback>
                </Avatar>
                <div className="grid flex-1 text-sm leading-tight text-left">
                  <span className="truncate">{user?.email}</span>
                </div>
              </div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem>
                <BadgeCheck className="mr-2 size-4" />
                Account
              </DropdownMenuItem>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={logout}>
              <LogOut className="mr-2 size-4" />
              Log out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
