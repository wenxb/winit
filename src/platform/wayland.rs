//! # Wayland
//!
//! **Note:** Windows don't appear on Wayland until you draw/present to them.
//!
//! By default, Winit loads system libraries using `dlopen`. This can be
//! disabled by disabling the `"wayland-dlopen"` cargo feature.
//!
//! ## Client-side decorations
//!
//! Winit provides client-side decorations by default, but the behaviour can
//! be controlled with the following feature flags:
//!
//! * `wayland-csd-adwaita` (default).
//! * `wayland-csd-adwaita-crossfont`.
//! * `wayland-csd-adwaita-notitle`.

use std::ffi::c_void;
use std::ptr::NonNull;

use crate::event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder};
use crate::monitor::MonitorHandle;
use crate::window::{Window, WindowAttributes};

pub use crate::window::Theme;

/// Additional methods on [`ActiveEventLoop`] that are specific to Wayland.
pub trait ActiveEventLoopExtWayland {
    /// True if the [`ActiveEventLoop`] uses Wayland.
    fn is_wayland(&self) -> bool;
}

impl ActiveEventLoopExtWayland for ActiveEventLoop {
    #[inline]
    fn is_wayland(&self) -> bool {
        self.p.is_wayland()
    }
}

/// Additional methods on [`EventLoop`] that are specific to Wayland.
pub trait EventLoopExtWayland {
    /// True if the [`EventLoop`] uses Wayland.
    fn is_wayland(&self) -> bool;
}

impl<T: 'static> EventLoopExtWayland for EventLoop<T> {
    #[inline]
    fn is_wayland(&self) -> bool {
        self.event_loop.is_wayland()
    }
}

/// Additional methods on [`EventLoopBuilder`] that are specific to Wayland.
pub trait EventLoopBuilderExtWayland {
    /// Force using Wayland.
    fn with_wayland(&mut self) -> &mut Self;

    /// Whether to allow the event loop to be created off of the main thread.
    ///
    /// By default, the window is only allowed to be created on the main
    /// thread, to make platform compatibility easier.
    fn with_any_thread(&mut self, any_thread: bool) -> &mut Self;
}

impl<T> EventLoopBuilderExtWayland for EventLoopBuilder<T> {
    #[inline]
    fn with_wayland(&mut self) -> &mut Self {
        self.platform_specific.forced_backend = Some(crate::platform_impl::Backend::Wayland);
        self
    }

    #[inline]
    fn with_any_thread(&mut self, any_thread: bool) -> &mut Self {
        self.platform_specific.any_thread = any_thread;
        self
    }
}

/// Additional methods on [`Window`] that are specific to Wayland.
///
/// [`Window`]: crate::window::Window
pub trait WindowExtWayland {
    /// Returns `xdg_toplevel` of the window or [`None`] if the window is X11 window.
    fn xdg_toplevel(&self) -> Option<NonNull<c_void>>;

    /// Sets the window geometry (visible bounds excluding shadows/margins for window manager snapping/tiling).
    fn set_window_geometry(&self, x: i32, y: i32, width: u32, height: u32);

    /// Sets the window geometry margins (left, top, right, bottom) excluding shadows for window manager snapping/tiling.
    fn set_geometry_margins(&self, left: u32, top: u32, right: u32, bottom: u32);
}

impl WindowExtWayland for Window {
    #[inline]
    fn xdg_toplevel(&self) -> Option<NonNull<c_void>> {
        #[allow(clippy::single_match)]
        match &self.window {
            #[cfg(x11_platform)]
            crate::platform_impl::Window::X(_) => None,
            #[cfg(wayland_platform)]
            crate::platform_impl::Window::Wayland(window) => window.xdg_toplevel(),
        }
    }

    #[inline]
    fn set_window_geometry(&self, x: i32, y: i32, width: u32, height: u32) {
        #[allow(clippy::single_match)]
        match &self.window {
            #[cfg(x11_platform)]
            crate::platform_impl::Window::X(_) => (),
            #[cfg(wayland_platform)]
            crate::platform_impl::Window::Wayland(window) => {
                window.set_window_geometry(x, y, width, height)
            }
        }
    }

    #[inline]
    fn set_geometry_margins(&self, left: u32, top: u32, right: u32, bottom: u32) {
        #[allow(clippy::single_match)]
        match &self.window {
            #[cfg(x11_platform)]
            crate::platform_impl::Window::X(_) => (),
            #[cfg(wayland_platform)]
            crate::platform_impl::Window::Wayland(window) => {
                window.set_geometry_margins(left, top, right, bottom)
            }
        }
    }
}

/// Sets window geometry for a Wayland window by its raw `wl_surface` pointer.
///
/// This is particularly useful for GUI toolkits (such as Iced) that provide
/// raw window handles instead of `winit::window::Window`.
pub fn set_window_geometry_for_surface_ptr(
    surface_ptr: *mut c_void,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> bool {
    #[cfg(wayland_platform)]
    {
        crate::platform_impl::wayland::set_geometry_for_surface_ptr(surface_ptr, x, y, width, height)
    }
    #[cfg(not(wayland_platform))]
    {
        let _ = (surface_ptr, x, y, width, height);
        false
    }
}

/// Sets window geometry margins (left, top, right, bottom) for a Wayland window by its raw `wl_surface` pointer.
///
/// The window manager will snap/tile the window excluding these margins.
pub fn set_geometry_margins_for_surface_ptr(
    surface_ptr: *mut c_void,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
) -> bool {
    #[cfg(wayland_platform)]
    {
        crate::platform_impl::wayland::set_geometry_margins_for_surface_ptr(surface_ptr, left, top, right, bottom)
    }
    #[cfg(not(wayland_platform))]
    {
        let _ = (surface_ptr, left, top, right, bottom);
        false
    }
}

/// Additional methods on [`WindowAttributes`] that are specific to Wayland.
pub trait WindowAttributesExtWayland {
    /// Build window with the given name.
    ///
    /// The `general` name sets an application ID, which should match the `.desktop`
    /// file distributed with your program. The `instance` is a `no-op`.
    ///
    /// For details about application ID conventions, see the
    /// [Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#desktop-file-id)
    fn with_name(self, general: impl Into<String>, instance: impl Into<String>) -> Self;
}

impl WindowAttributesExtWayland for WindowAttributes {
    #[inline]
    fn with_name(mut self, general: impl Into<String>, instance: impl Into<String>) -> Self {
        self.platform_specific.name =
            Some(crate::platform_impl::ApplicationName::new(general.into(), instance.into()));
        self
    }
}

/// Additional methods on `MonitorHandle` that are specific to Wayland.
pub trait MonitorHandleExtWayland {
    /// Returns the inner identifier of the monitor.
    fn native_id(&self) -> u32;
}

impl MonitorHandleExtWayland for MonitorHandle {
    #[inline]
    fn native_id(&self) -> u32 {
        self.inner.native_identifier()
    }
}
