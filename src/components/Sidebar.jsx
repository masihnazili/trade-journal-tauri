import React from 'react';
import { NavLink } from 'react-router-dom';

export default function Sidebar() {
  const linkClass = ({ isActive }) => 'nav-item' + (isActive ? ' active' : '');
  return (
    <aside className="sidebar">
      <div className="logo">ژورنال معاملات</div>
      <nav>
        <NavLink to="/" end className={linkClass}>داشبورد</NavLink>
        <NavLink to="/trades" className={linkClass}>لیست معاملات</NavLink>
        <NavLink to="/trades/new" className={linkClass}>ثبت معامله جدید</NavLink>
        <NavLink to="/calendar" className={linkClass}>تقویم معاملاتی</NavLink>
        <NavLink to="/settings" className={linkClass}>تنظیمات</NavLink>
      </nav>
    </aside>
  );
}
